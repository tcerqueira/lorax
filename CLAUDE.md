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

114 tree-walk tests are `#[ignore]` for unimplemented features (classes, `this`, `super`, some resolver checks). The VM test binary **shadows** `lox_tests!` locally to slap `#[ignore = "VM not yet implemented"]` on every case — the VM doesn't yet cover enough of Lox to share the test suite, so remove the shadow as features land.

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

- **Single-pass Pratt parser → bytecode** (`vm/src/compiler.rs`): no intermediate AST. `parse_bp` recurses by binding power; prefix/infix/postfix dispatch tables (`prefix_bp`/`infix_bp`/`postfix_bp`) gate the loop. Errors trigger `synchronize()` to skip to the next statement boundary so a single bad token doesn't poison the whole compile.
- **`Handle` lvalue/rvalue threading**: `parse_*` methods return `Handle::Value` (already on the stack) or `Handle::Place` (deferred — emit `SetGlobal`/etc. on materialize). This is how `=` and global stores will be wired without a second pass.
- **Encoded opcodes** (`vm/src/enconding.rs`): `OpCode` is `#[repr(u8)]` with inline operands; `Encode`/`Decode` traits serialize to/from the chunk's byte buffer. Adding an opcode means updating both arms plus the VM's `match`.
- **Custom heap** (`vm/src/storage.rs`, `vm/src/object.rs`): `Storage` owns an `ObjectPool` (intrusive `SinglyLinkedList` of type-erased `UnsafeRef<Object>`) plus a `lasso::Rodeo` string interner. `Object` is a `#[repr(C)]` header with a `kind` tag; the concrete type (`LoxString`) is downcast unsafely from the tag. `OwnedObject::drop` dispatches on `kind` to free the correct DST layout — `Box<Object>` alone can't, because the alloc is oversized.
- **`Value::Symbol(Spur)` for identifiers, `LoxString` for runtime strings**: identifiers are interned in `Storage`'s `Rodeo` and live inline in `Value` as `Spur` keys (cheap equality, used for globals lookup); runtime strings (e.g. concat results) are heap `LoxString` reached through `Value::Object`. The VM's `equal` op routes through `Value::as_str` so a `Symbol` and a `LoxString` with the same contents compare equal.

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

Walking the book's "Compiling Expressions" / "Global Variables" chapters. Implemented: literals (`true`/`false`/`nil`/numbers/strings), unary `-`/`!`, all arithmetic/comparison/equality ops, string concatenation, `print`, expression statements, global `var` declarations and reads, error reporting + `synchronize()`. **`SetGlobal` (assignment) is still `todo!()`** — and there are no locals, control flow, functions, or classes yet, so the VM cannot run most programs in `tests/sources/`.

## Toolchain

- **Nightly Rust** required (`rust-toolchain.toml`). Features in use across crates: `formatting_options`, `error_iter`; VM additionally uses `ptr_metadata`, `arbitrary_self_types`, `if_let_guard`.
- **Edition 2024**.
