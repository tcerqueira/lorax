#!/usr/bin/env -S deno run -A
// Convert a samply `--save-only` Firefox-Profiler JSON (`*.json.gz`) into folded
// stacks + top self/total tables, with the executable's raw addresses
// symbolicated offline (samply does symbolication lazily in its UI, so the
// save-time JSON carries only library-relative addresses for the exe).
//
// Usage:
//   deno run -A scripts/samply-folded.ts prof.json.gz \
//       [--bin ./target/release/rlox] [--top 40] [> folded.txt]
//
// Folded stacks go to stdout (pipe to inferno-flamegraph for an SVG); the
// self/total symbol tables go to stderr. See the project_profiling_samply
// memory for the end-to-end workflow.

const args = Deno.args.slice();
function flag(name: string, def?: string): string | undefined {
  const i = args.indexOf(name);
  if (i >= 0) {
    const v = args[i + 1];
    args.splice(i, 2);
    return v;
  }
  return def;
}

const bin = flag("--bin", "./target/release/rlox")!;
const topN = parseInt(flag("--top", "40")!, 10);
const TEXT_VMADDR = BigInt(flag("--vmaddr", "0x100000000")!);
const input = args[0];
if (!input) {
  console.error("usage: samply-folded.ts <prof.json.gz> [--bin BIN] [--top N]");
  Deno.exit(2);
}

// ---- load + gunzip ---------------------------------------------------------
const raw = await Deno.readFile(input);
let jsonText: string;
if (input.endsWith(".gz")) {
  const ds = new DecompressionStream("gzip");
  const stream = new Blob([raw]).stream().pipeThrough(ds);
  jsonText = await new Response(stream).text();
} else {
  jsonText = new TextDecoder().decode(raw);
}
const profile = JSON.parse(jsonText);

const libs: Array<{ name?: string; debugName?: string; path?: string }> =
  profile.libs ?? [];
const exeLibIdx = new Set<number>();
libs.forEach((l, i) => {
  const n = l.name ?? l.debugName ?? l.path ?? "";
  if (n === bin.split("/").pop() || (l.path && l.path.endsWith(bin.replace(/^\.\//, "")))) {
    exeLibIdx.add(i);
  }
});
// Fallback: if nothing matched, treat the lib whose name has no extension and
// matches the binary's basename loosely.
if (exeLibIdx.size === 0) {
  const base = bin.split("/").pop()!;
  libs.forEach((l, i) => {
    if ((l.name ?? l.debugName ?? "").includes(base)) exeLibIdx.add(i);
  });
}

// ---- gather frames, resolve symbols ----------------------------------------
type FrameInfo = { name: string; exeOffset: bigint | null };
const frameCache = new Map<string, FrameInfo[]>(); // per-thread keyed cache

// Collect exe offsets across all threads first so we can batch-symbolicate.
const exeOffsets = new Set<bigint>();

interface Thread {
  name?: string;
  samples: { stack: Array<number | null>; weight?: number[]; length?: number };
  stackTable: { prefix: Array<number | null>; frame: number[]; length: number };
  frameTable: { address: number[]; func: number[]; length: number };
  funcTable: { name: number[]; resource: number[]; length: number };
  resourceTable?: { lib: number[]; name: number[]; length: number };
  stringArray?: string[];
  stringTable?: { _array?: string[] } | string[];
}

const threads: Thread[] = profile.threads ?? [];

function strings(t: Thread): string[] {
  if (Array.isArray(t.stringArray)) return t.stringArray;
  if (Array.isArray(t.stringTable)) return t.stringTable as string[];
  if (t.stringTable && Array.isArray((t.stringTable as any)._array)) {
    return (t.stringTable as any)._array;
  }
  return [];
}

function frameExeOffset(t: Thread, frameIdx: number): bigint | null {
  const addr = t.frameTable.address[frameIdx];
  if (addr == null || addr < 0) return null;
  const funcIdx = t.frameTable.func[frameIdx];
  const resIdx = t.funcTable.resource[funcIdx];
  const libIdx = resIdx != null && resIdx >= 0 && t.resourceTable
    ? t.resourceTable.lib[resIdx]
    : -1;
  if (exeLibIdx.has(libIdx)) return BigInt(addr);
  return null;
}

for (const t of threads) {
  for (let f = 0; f < t.frameTable.length; f++) {
    const off = frameExeOffset(t, f);
    if (off != null) exeOffsets.add(off);
  }
}

// ---- batch atos + rustfilt -------------------------------------------------
async function runCapture(cmd: string, cmdArgs: string[], stdin?: string): Promise<string> {
  const c = new Deno.Command(cmd, {
    args: cmdArgs,
    stdin: stdin != null ? "piped" : "null",
    stdout: "piped",
    stderr: "piped",
  });
  const child = c.spawn();
  if (stdin != null) {
    const w = child.stdin.getWriter();
    await w.write(new TextEncoder().encode(stdin));
    await w.close();
  }
  const { stdout } = await child.output();
  return new TextDecoder().decode(stdout);
}

const offsetSymbol = new Map<bigint, string>();
if (exeOffsets.size > 0) {
  const offsets = [...exeOffsets];
  const addrArgs = offsets.map((o) => "0x" + (TEXT_VMADDR + o).toString(16));
  // atos: one symbol line per input address, in order.
  let atosOut: string;
  try {
    atosOut = await runCapture("atos", [
      "-o", bin, "-arch", "arm64", "-l", "0x" + TEXT_VMADDR.toString(16), ...addrArgs,
    ]);
  } catch (e) {
    console.error("atos failed:", e);
    atosOut = offsets.map((o) => "0x" + o.toString(16)).join("\n");
  }
  // Demangle the whole block through rustfilt at once.
  let demangled = atosOut;
  try {
    const rustfilt = (await runCapture("which", ["rustfilt"])).trim() ||
      `${Deno.env.get("HOME")}/.cargo/bin/rustfilt`;
    demangled = await runCapture(rustfilt, [], atosOut);
  } catch { /* leave as-is */ }
  const lines = demangled.split("\n");
  offsets.forEach((o, i) => {
    let s = (lines[i] ?? "").trim();
    // Strip the trailing "(in rlox) (file.rs:line)" noise, keep symbol + line.
    s = s.replace(/\s+\(in [^)]+\)/, "");
    if (!s) s = "0x" + o.toString(16);
    offsetSymbol.set(o, s);
  });
}

function frameName(t: Thread, frameIdx: number): string {
  const off = frameExeOffset(t, frameIdx);
  if (off != null) return offsetSymbol.get(off) ?? "0x" + off.toString(16);
  const funcIdx = t.frameTable.func[frameIdx];
  const strs = strings(t);
  const nm = strs[t.funcTable.name[funcIdx]] ?? "?";
  // Non-exe frame: prefix with lib name if it is a bare address.
  const resIdx = t.funcTable.resource[funcIdx];
  const libIdx = resIdx != null && resIdx >= 0 && t.resourceTable
    ? t.resourceTable.lib[resIdx]
    : -1;
  const libName = libIdx >= 0 ? (libs[libIdx]?.name ?? libs[libIdx]?.debugName ?? "?") : "?";
  if (/^0x[0-9a-f]+$/i.test(nm)) return `${libName}\`${nm}`;
  return nm;
}

// ---- walk samples → folded stacks ------------------------------------------
const folded = new Map<string, number>();
const selfTime = new Map<string, number>();
const totalTime = new Map<string, number>();

for (const t of threads) {
  // Resolve every stack node's name once.
  const nameOf: string[] = new Array(t.stackTable.length);
  for (let s = 0; s < t.stackTable.length; s++) {
    nameOf[s] = frameName(t, t.stackTable.frame[s]);
  }
  const weights = t.samples.weight;
  const stacks = t.samples.stack;
  for (let i = 0; i < stacks.length; i++) {
    let node = stacks[i];
    if (node == null) continue;
    const w = weights ? weights[i] : 1;
    // Build root→leaf chain.
    const chain: string[] = [];
    let n: number | null = node;
    while (n != null) {
      chain.push(nameOf[n]);
      n = t.stackTable.prefix[n];
    }
    chain.reverse();
    const key = chain.join(";");
    folded.set(key, (folded.get(key) ?? 0) + w);
    // self = leaf, total = each unique symbol in chain.
    const leaf = chain[chain.length - 1];
    selfTime.set(leaf, (selfTime.get(leaf) ?? 0) + w);
    const seen = new Set<string>();
    for (const s of chain) {
      if (seen.has(s)) continue;
      seen.add(s);
      totalTime.set(s, (totalTime.get(s) ?? 0) + w);
    }
  }
}

// ---- emit ------------------------------------------------------------------
const foldedLines = [...folded.entries()]
  .sort((a, b) => b[1] - a[1])
  .map(([k, v]) => `${k} ${v}`);
console.log(foldedLines.join("\n"));

const totalSamples = [...selfTime.values()].reduce((a, b) => a + b, 0) || 1;
function table(title: string, m: Map<string, number>) {
  console.error(`\n=== ${title} (of ${totalSamples} samples) ===`);
  const rows = [...m.entries()].sort((a, b) => b[1] - a[1]).slice(0, topN);
  for (const [name, v] of rows) {
    const pct = ((v / totalSamples) * 100).toFixed(1).padStart(5);
    console.error(`${pct}%  ${String(v).padStart(7)}  ${name}`);
  }
}

// Aggregate per FUNCTION by stripping the trailing " + <offset>" atos appends
// when an address has no source-line mapping. This collapses every hot
// instruction inside a function into one row.
function byFunc(m: Map<string, number>): Map<string, number> {
  const out = new Map<string, number>();
  for (const [name, v] of m) {
    const fn = name.replace(/\s+\+\s*\d+$/, "").replace(/\s+\([^)]*\.rs:\d+\)$/, "");
    out.set(fn, (out.get(fn) ?? 0) + v);
  }
  return out;
}
table("SELF time by FUNCTION", byFunc(selfTime));
table("TOTAL time by FUNCTION", byFunc(totalTime));
table("SELF time by instruction site", selfTime);
