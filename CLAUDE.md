# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**lorax** is a Rust implementation of the Lox programming language from "Crafting Interpreters" by Robert Nystrom. The original Java/C implementation and language specification live at https://github.com/munificent/craftinginterpreters. It has two backends: a **tree-walk interpreter** and a **bytecode VM**. The VM implements the full Lox language (functions, closures, classes, inheritance, `super`) plus a mark-sweep garbage collector; the tree-walk backend stops short of classes.

## Build & Run Commands

```bash
cargo build                                # Dev build
cargo build --release                      # Release build
cargo check                               # Type-check only
cargo clippy                               # Lint

cargo run -- script.lox                    # Run a Lox script (tree-walk)
cargo run -- --vm script.lox               # Run with VM backend
cargo run -- -c 'print 1 + 2;'             # Run inline source (tree-walk)
cargo run -- --vm -c 'print 1 + 2;'        # Run inline source (VM)
cargo run                                  # Interactive REPL
```

## Testing

Tests use a `lox_tests!` macro (`src/test_utils.rs`) that runs `.lox` files from `tests/sources/` and checks output against special comments (`// expect: <output>`, `// expect runtime error:`, `// Error at`). Each backend has its own test binary (`tests/tree_walk.rs`, `tests/vm.rs`).

```bash
cargo test                                          # All tests
cargo test --test <backend>                         # All tests for a backend
cargo test --test <backend> <module>::              # Single test module
cargo test --test <backend> <module>::<test_name>   # Single test
```

114 tree-walk tests are `#[ignore]` for unimplemented features (classes, `this`, `super`, some resolver checks). The VM (`tests/vm.rs`) implements the full language; only 6 cases stay `#[ignore]`, all intentional semantic deviations from the book (same-scope shadowing is legal, so `var a = a + 1;` rebinds the previous `a`; constants are deduplicated; there is no AST-dump `parse` mode). Run the whole VM suite with `LORAX_STRESS_GC=1` to collect on every instruction — the strongest GC check.

## Benchmarks

Criterion benchmarks live in `benches/benchmarks.rs` (a `harness = false` bench target of the root `rlox` crate); the Lox programs are in `benches/sources/`. The harness drives both backends in-process (no subprocess), embedding each source via `include_str!`. Sources split into two sets in the harness: `PORTABLE` (uses only the shared feature subset — runs on both backends) and `VM_ONLY` (classes/inheritance — VM only). Three groups:

- `compile` — source → `Chunk` (VM single-pass compiler), every program.
- `vm` — full compile+execute on the VM, every program.
- `tree_walk` — full scan→parse→resolve→interpret on the tree-walk backend, portable programs only.

To compare a backend on a program, line up `vm/<name>` against `tree_walk/<name>`; both time the whole source-string-to-result pipeline on a freshly built interpreter, so they're symmetric.

```bash
cargo bench --bench benchmarks                       # Everything (~minutes)
cargo bench --bench benchmarks -- 'tree_walk|vm'     # Just the backend comparison
cargo bench --bench benchmarks -- fib                # One program, all groups
```

The sources are adapted from the book's `test/benchmark/` suite plus a few new ones (`closures`, `string_concat`, `mutual_recursion`, `arithmetic`, `loops`): `clock()`/`print` self-timing is stripped, and the portable workloads are sized so the *slower* tree-walk backend lands in the tens-of-ms range (the VM then runs them in a few ms). The book's time-based `zoo_batch` is omitted (incompatible with criterion). As of the last run the VM is ~3.5–6.6× faster than tree-walk across the portable programs. Do **not** set `LORAX_STRESS_GC` while benchmarking. Results land in `target/criterion/` (gitignored).

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `rlox` (root) | CLI entry point, test infrastructure (`lox_tests!` macro) |
| `lexer` | Tokenizer/scanner, shared by both backends |
| `report` | Error types (lexing, parsing, runtime) and source-span reporting |
| `tree-walk` | Tree-walk interpreter: parser, resolver, interpreter |
| `vm` | Bytecode VM: single-pass Pratt-parsing compiler, stack-based interpreter, custom heap |

## Architecture

### Execution Pipeline (tree-walk)

```
Source → Scanner (lexer) → Parser → AST → Resolver → Interpreter → Output
```

Key flow in `tree-walk/src/lib.rs`:
1. `Scanner::new(&source).scan_tokens()` — tokenizes source
2. `Parser::new(arena, tokens).parse()` — recursive descent into AST
3. `Resolver::new(interpreter, arena).resolve(&program)` — variable scope resolution
4. `interpreter.interpret(program, arena)` — evaluates AST

### Tree-walk design patterns

- **AST arena allocation** (`tree-walk/src/parsing/ast.rs`): All AST nodes live in a `SlotMap`-backed `AstArena`. References are `ExprRef<'a>`/`StmtRef<'a>` smart pointers, avoiding `Box<Expr>` trees.
- **Visitor pattern** (`tree-walk/src/parsing/visitor.rs`): `ExprVisitor`/`StmtVisitor` traits implemented by `Interpreter` and `Resolver`.
- **Environment chain** (`tree-walk/src/runtime/environment.rs`): Scopes are a linked list (`Chain<T>`). Closures clone the chain to capture enclosing environments.
- **Type-erased objects** (`tree-walk/src/runtime/object.rs`): `Object` wraps `Rc<dyn ObjectInternal>` supporting f64, String, bool, nil, Function, NativeFunction with runtime downcast.
- **Control flow as values** (`tree-walk/src/runtime/control_flow.rs`): `Return`, `Break`, `Continue` propagate via a `ControlFlow` enum rather than exceptions.

### VM design patterns

- **Single-pass Pratt parser → bytecode** (`vm/src/compiler.rs`): no intermediate AST. `parse_bp` recurses by binding power; prefix/infix/postfix dispatch tables (`prefix_bp`/`infix_bp`/`postfix_bp`) gate the loop. Errors trigger `synchronize()` to skip to the next statement boundary so a single bad token doesn't poison the whole compile. The `Compiler` is a thin driver over a stack of per-function `Target`s (each owns a `Chunk` + `Scopes` + `FunctionKind`); compiling a `fun`/method pushes a `Target` and materializes a `LoxFunction` when its body ends. A parallel `classes` stack makes `this`/`super` legality structural.
- **`Handle` lvalue/rvalue threading**: `parse_*` methods return `Handle::Value` (already on the stack) or `Handle::Place` (a deferred lvalue: global, local, upvalue, or instance property). `materialize` turns a `Place` into a read (`Get*`); `store` (driven by `assignment` on `=`) turns it into a write (`Set*`). This wires `=`, variable stores, and `a.b = c` without a second pass.
- **Jump backpatching** (`vm/src/compiler.rs`): forward jumps (`Jmp`/`JmpIfFalse`) are emitted with a placeholder `0` offset and fixed up by `patch_jmp` once the target is known; backward jumps (`while`/`for`) use `emit_loop`. Over-long jumps/loops report a recoverable compile error rather than panicking. At runtime a jump is `frame.ip += offset` / `-= offset` (byte-offset IP, no cursor).
- **Encoded opcodes** (`vm/src/enconding.rs`): `OpCode` is `#[repr(u8)]` with inline operands; the compiler `Encode`s into the chunk's byte buffer, and both the VM dispatch loop and the disassembler decode via the single `OpCode::decode_at(&[u8], &mut ip)`. Adding an opcode means updating the enum, the `Encode` arm, the `decode_at` arm, the `disassemble` arm, and the VM's `match`.
- **CallFrame stack + flat dispatch** (`vm/src/vm.rs`, `vm/src/vm/frame.rs`): execution is a flat loop over a `Vec<CallFrame>`; `Call` pushes a frame, `Ret` pops one, an empty stack halts. The top-level script runs as `FrameSource::TopLevel(Chunk)` (frame 0); a call runs as `FrameSource::Closure(UnsafeRef<Object>)` reached through the heap handle (never a tracked `&Chunk`, since the callee's body mutates the heap that owns its chunk). Locals are `base + slot`; a `FRAMES_MAX` cap gives a graceful "Stack overflow." Recursion never rides the Rust stack.
- **Closures via stack-index upvalues** (`vm/src/object/{closure,upvalue}.rs`): `resolve_upvalue` walks the enclosing `Target` stack; `OP_CLOSURE` carries an `(is_local, index)` tail. A `LoxUpvalue` is `Open(stack index)` while its variable is live and `Closed(Value)` after — an index, not a raw pointer, so the reallocating value stack can't dangle it. The VM keeps a `Vec` of open upvalues so sibling closures share one; `OP_CLOSE_UPVALUE` / frame return hoist captured locals into their own cell.
- **Classes through the same machinery** (`vm/src/object/{class,instance,bound_method}.rs`): `.` is an infix operator yielding a `Place::Property`, so get/set reuse `materialize`/`store`; `recv.m(args)` fuses into `OP_INVOKE`. Methods live in a `RefCell<SymbolMap<Value>>` on `LoxClass`; `init`, `this` (slot 0), bound methods, and copy-down inheritance (`OP_INHERIT`) follow clox. `super` is a synthetic upvalue captured from the class-declaration scope.
- **Custom heap + mark-sweep GC** (`vm/src/storage.rs`, `vm/src/object.rs`, `vm/src/gc.rs`): `Storage` owns an `ObjectPool` (intrusive `SinglyLinkedList` of type-erased `UnsafeRef<Object>`) plus a `lasso::Rodeo` interner. `Object` is a `#[repr(C)]` header (`kind` tag + GC `mark` `Cell` + list link); concrete types downcast unsafely from the tag, and `OwnedObject::drop` dispatches on `kind` to free the correct (possibly DST) layout. The collector marks roots (stack, globals, frame closures, open upvalues, the script chunk's constants), blackens via `gc::Tracer`, and sweeps the intrusive list. Triggered at a dispatch safe point by a live-object threshold; `LORAX_STRESS_GC=1` (or `VirtualMachine::stress()`) collects every instruction.
- **`Value::Symbol(Spur)` for identifiers, `LoxString` for runtime strings**: identifiers, string literals, and member names are interned in `Storage`'s `Rodeo` and live inline in `Value` as `Spur` keys (cheap equality; keys for globals/fields/methods, which are `FxHashMap`s). Runtime strings (concat results) are heap `LoxString` reached through `Value::Object`; the interner is permanent and never collected. The VM's `equal` op routes through `Value::as_str` so a `Symbol` and a `LoxString` with the same contents compare equal.

## Lox Implementation Status

### Implemented (tree-walk)

- Datatypes: numbers (f64), strings, booleans, nil
- Variables: `var` declarations, block scoping, closures
- Control flow: `if`/`else`, `while`, `for`, `break`, `continue`
- Functions: first-class, closures, recursion, `return`
- Operators: arithmetic, comparison, equality, logical (`and`/`or`), unary (`-`, `!`)
- `print` statement
- Native function: `clock()`
- REPL

### Not implemented (tree-walk)

- Classes, instances, methods, properties
- `this` keyword
- Inheritance, `super` keyword
- Resolver strictness: duplicate local/parameter detection, self-referencing initializer detection

### VM status

The VM implements the **full Lox language** — every book chapter through "Optimization" (ch. 17–30):

- Datatypes, all operators, `print`, globals, locals with block scoping/shadowing, `if`/`else`, logical `and`/`or`, `while`, `for`
- Functions: declarations, first-class values, recursion, `return`, arity checking, the native `clock()`
- Closures: upvalues with sharing and close-on-scope-exit
- Classes: instances, fields, methods, `init`, `this`, bound methods, `OP_INVOKE`
- Inheritance (`<`), copy-down methods, `super` access and calls
- Mark-sweep garbage collection (`gc::Tracer` + intrusive sweep list)
- Graceful compile/runtime limits (too many constants/locals/upvalues/args, oversized jumps, stack overflow)

Optimizations applied (from ch. 30 and the perf review): `FxHashMap` for `Spur`-keyed tables, raw-byte instruction dispatch (`decode_at`, no per-op enum/cursor), single-allocation string concat, compile-time string interning. NaN-boxing and a hand-rolled probe table were deliberately rejected as poor fits for the Rust `Value` enum.

All non-skipped `tests/sources/` cases pass on the VM; the only `#[ignore]`s are intentional semantic deviations (see Testing). The new unsafe paths (object downcasts/drops, closures, GC sweep) are covered by in-process end-to-end tests in `vm/src/lib.rs` that run under Miri and stress GC.

## Toolchain

- **Nightly Rust** required (`rust-toolchain.toml`). The only unstable features in use are `ptr_metadata` and `arbitrary_self_types`, both in the `vm` crate.
- **Edition 2024**.
