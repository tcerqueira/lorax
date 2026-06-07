//! Criterion benchmarks comparing the two Lox backends.
//!
//! Each `.lox` program under `benches/sources/` exercises a different part of an
//! interpreter (recursion, closures, allocation/GC, method dispatch, field
//! access, dynamic equality, string building, raw arithmetic, loop overhead).
//! They split into two sets:
//!
//! * [`PORTABLE`] — uses only the feature subset both backends implement, so each
//!   runs on **tree-walk and VM** for a head-to-head comparison.
//! * [`VM_ONLY`] — uses classes/inheritance, which the tree-walk backend does not
//!   implement, so these run on the **VM** only.
//!
//! Three groups:
//!
//! * `compile`    — source → [`Chunk`] (VM single-pass compiler), every program.
//! * `vm`         — full compile+execute on the VM, every program.
//! * `tree_walk`  — full scan→parse→resolve→interpret on the tree-walk backend,
//!   the portable programs only.
//!
//! To compare a backend on a program, line up `vm/<name>` against
//! `tree_walk/<name>`; both groups time the whole source-string-to-result
//! pipeline — each constructs its own interpreter and runs compile + execute
//! inside the timed region — so the comparison is structurally apples-to-apples.
//!
//! The sources are adapted from the Crafting Interpreters benchmark suite (plus
//! a few new ones): their `clock()`/`print` self-timing is stripped and the
//! portable workloads are sized so the *slower* tree-walk backend lands in the
//! tens-of-ms range (the VM then runs them in a few ms). Programs produce no
//! output, so `cargo bench` stays quiet.
//!
//! Run everything with `cargo bench --bench benchmarks`, just the comparison with
//! `cargo bench --bench benchmarks -- 'tree_walk|vm'`, or one program with
//! `cargo bench --bench benchmarks -- fib`. Do **not** set `LORAX_STRESS_GC`
//! while benchmarking — it collects on every instruction and is orders of
//! magnitude slower.

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use lexer::Scanner;
use report::Reporter;
use vm::{chunk::Chunk, compiler::Compiler, vm::VirtualMachine};

/// Programs in the shared feature subset — run on both backends.
const PORTABLE: &[(&str, &str)] = &[
    ("fib", include_str!("sources/fib.lox")),
    ("mutual_recursion", include_str!("sources/mutual_recursion.lox")),
    ("closures", include_str!("sources/closures.lox")),
    ("arithmetic", include_str!("sources/arithmetic.lox")),
    ("loops", include_str!("sources/loops.lox")),
    ("equality", include_str!("sources/equality.lox")),
    ("string_equality", include_str!("sources/string_equality.lox")),
    ("string_concat", include_str!("sources/string_concat.lox")),
];

/// Class-based programs — VM only (the tree-walk backend has no classes).
const VM_ONLY: &[(&str, &str)] = &[
    ("binary_trees", include_str!("sources/binary_trees.lox")),
    ("trees", include_str!("sources/trees.lox")),
    ("zoo", include_str!("sources/zoo.lox")),
    ("method_call", include_str!("sources/method_call.lox")),
    ("invocation", include_str!("sources/invocation.lox")),
    ("properties", include_str!("sources/properties.lox")),
    ("instantiation", include_str!("sources/instantiation.lox")),
];

/// Every benchmark program (portable first); the `compile` and `vm` groups run
/// the whole set on the VM.
fn all_programs() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    PORTABLE.iter().chain(VM_ONLY)
}

/// Compile `source` into a [`Chunk`], interning into `vm`'s storage (which the
/// resulting chunk references). Panics on a compile error: benchmark sources are
/// expected to be valid.
fn compile(source: &str, vm: &mut VirtualMachine) -> Chunk {
    let reporter = Reporter::new(source);
    let scanner = Scanner::new(source);
    let mut compiler = Compiler::new(scanner, reporter, vm.storage());
    compiler
        .compile()
        .expect("benchmark source should compile cleanly")
}

/// VM single-pass compiler throughput per program. Each compile needs its own
/// fresh VM — the compiler both interns into and materializes function objects
/// into the VM's storage, so reusing one VM would accumulate objects across
/// iterations. The fresh VM is built in setup (excluded from the timed region),
/// so only `compile` is measured. `PerIteration` keeps exactly one of these
/// heavyweight VMs alive at a time; a batched `BatchSize` would pre-construct a
/// whole batch (~iters/10 for `SmallInput`) of them up front and blow memory,
/// since the routine is only microseconds and `iters` runs into the millions.
fn bench_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");
    for &(name, source) in all_programs() {
        group.bench_function(name, |b| {
            b.iter_batched(
                VirtualMachine::new,
                |mut vm| black_box(compile(black_box(source), &mut vm)),
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

/// Full VM pipeline per program: construct a VM, compile, and execute. Only the
/// source-string clone is excluded from the timed region.
fn bench_vm(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm");
    for &(name, source) in all_programs() {
        group.bench_function(name, |b| {
            b.iter_batched(
                || source.to_string(),
                |src| {
                    let mut vm = VirtualMachine::new();
                    vm::run(src, &mut vm).expect("benchmark should run without error");
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Full tree-walk pipeline per portable program: construct an interpreter, scan,
/// parse, resolve, and interpret. Symmetric to [`bench_vm`] — only the
/// source-string clone is excluded from the timed region.
fn bench_tree_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_walk");
    for &(name, source) in PORTABLE {
        group.bench_function(name, |b| {
            b.iter_batched(
                || source.to_string(),
                |src| tree_walk::run_source(src).expect("benchmark should run without error"),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    // 60 samples keeps the tens-of-ms cases snappy while staying statistically
    // meaningful; override on the CLI (e.g. `--sample-size 100`).
    config = Criterion::default().sample_size(60);
    targets = bench_compile, bench_vm, bench_tree_walk
}
criterion_main!(benches);
