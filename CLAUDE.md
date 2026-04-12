# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**lorax** is a Rust implementation of the Lox programming language from "Crafting Interpreters" by Robert Nystrom. The original Java/C implementation and language specification live at https://github.com/munificent/craftinginterpreters. It has two backends: a working **tree-walk interpreter** and an in-progress **bytecode VM**.

## Build & Run Commands

```bash
cargo build                                # Dev build
cargo build --release                      # Release build
cargo check                               # Type-check only
cargo clippy                               # Lint

cargo run -- script.lox                    # Run a Lox script (tree-walk)
cargo run -- --vm script.lox               # Run with VM backend (incomplete)
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

~114 tests are `#[ignore]` for unimplemented features (classes, `this`, `super`, some resolver checks). VM tests are all ignored.

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `rlox` (root) | CLI entry point, test infrastructure (`lox_tests!` macro) |
| `lexer` | Tokenizer/scanner, shared by both backends |
| `report` | Error types (lexing, parsing, runtime) and source-span reporting |
| `tree-walk` | Tree-walk interpreter: parser, resolver, interpreter |
| `vm` | Bytecode VM: compiler (mostly `todo!()`), opcodes, stack VM |

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

### Key Design Patterns

- **AST arena allocation** (`tree-walk/src/parsing/ast.rs`): All AST nodes live in a `SlotMap`-backed `AstArena`. References are `ExprRef<'a>`/`StmtRef<'a>` smart pointers, avoiding `Box<Expr>` trees.
- **Visitor pattern** (`tree-walk/src/parsing/visitor.rs`): `ExprVisitor`/`StmtVisitor` traits implemented by `Interpreter` and `Resolver`.
- **Environment chain** (`tree-walk/src/runtime/environment.rs`): Scopes are a linked list (`Chain<T>`). Closures clone the chain to capture enclosing environments.
- **Type-erased objects** (`tree-walk/src/runtime/object.rs`): `Object` wraps `Rc<dyn ObjectInternal>` supporting f64, String, bool, nil, Function, NativeFunction with runtime downcast.
- **Control flow as values** (`tree-walk/src/runtime/control_flow.rs`): `Return`, `Break`, `Continue` propagate via a `ControlFlow` enum rather than exceptions.

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

### Not implemented

- Classes, instances, methods, properties
- `this` keyword
- Inheritance, `super` keyword
- Resolver strictness: duplicate local/parameter detection, self-referencing initializer detection
- Bytecode VM compiler (opcodes and VM skeleton exist, compiler is stubbed with `todo!()`)

## Toolchain

- **Nightly Rust** required (`rust-toolchain.toml`) — uses `#![feature(formatting_options)]` and `#![feature(error_iter)]`.
- **Edition 2024**.
