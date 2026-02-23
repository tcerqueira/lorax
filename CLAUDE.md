# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lorax is a Lox programming language implementation in Rust with two interpreter backends: a feature-complete tree-walk interpreter and an in-progress bytecode VM.

## Build & Test Commands

```bash
cargo build                    # Build all crates
cargo test --lib               # Run all unit tests (22 tests)
cargo test --lib -p rlox-tree-walk  # Run tests for a specific crate
cargo test --lib test_name     # Run a single test by name
cargo run                      # Start REPL
cargo run -- examples/file.lox # Run a Lox script
```

**Toolchain:** Rust nightly (edition 2024), uses unstable features (`formatting_options`, `error_iter`).

**Dev environment:** Nix flake + direnv. Running `nix develop` or entering the directory with direnv loads the environment.

## Workspace Crates

- **rlox** — CLI binary. REPL mode (no args) or script execution (one arg). Currently wired to the tree-walk interpreter (VM is commented out).
- **rlox-lexer** — Scanner that implements `Iterator` to produce tokens from source text.
- **rlox-tree-walk** — Complete tree-walk interpreter with parsing, scope resolution, and runtime.
- **rlox-vm** — Bytecode VM (early stage). Scanner works, compiler has `todo!()` stubs.
- **rlox-report** — Shared error types and reporting across all compilation phases (lexing, parsing, runtime, passes).

## Architecture

**AST uses arena allocation:** Nodes are stored in a `SlotMap` and referenced by `ExprId`/`StmtId` handles (see `rlox-tree-walk/src/parsing/ast.rs`).

**Visitor pattern for evaluation:** `ExprVisitor` and `StmtVisitor` traits in `rlox-tree-walk/src/parsing/visitor.rs` are implemented by both the interpreter and the resolver pass.

**Scope resolution:** A resolver pass (`rlox-tree-walk/src/passes/resolver.rs`) runs before interpretation to compute variable binding depths.

**Environment chain:** Lexical scoping via a linked list of environments (`rlox-tree-walk/src/runtime/environment.rs`, `chain.rs`).

**Dynamic typing:** Runtime values use a trait-based object system (`ObjectInternal` trait in `rlox-tree-walk/src/runtime/object.rs`).

**Parser:** Recursive descent with 14 production rules in `rlox-tree-walk/src/parsing/parser.rs`. Expressions include Binary, Call, Grouping, Literal, Unary, Variable, Assign, Logical. Statements include Print, Var, Block, If, While, Function, Return.

## Lox Language Status

**Implemented:** variables, arithmetic, comparisons, logical operators, functions, closures, control flow (if/else, while, for), return, print, comments, lexical scoping.

**Not yet implemented:** classes, `this`, `super` (tokens are defined in the lexer).
