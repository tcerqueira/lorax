use std::{fmt::Display, iter::Peekable};

use anyhow::{Context, bail};
use lasso::Spur;
use lexer::{
    Scanner,
    tokens::{Token, TokenType},
};
use report::{Reporter, error::ParsingError};
use report::{Span, error::LexingError};
use scopeguard::ScopeGuard;

use crate::{
    chunk::Chunk,
    compiler::{error::CompileError, scopes::Scopes},
    enconding::{OpCode, Slot},
    object::function::LoxFunction,
    storage::Storage,
    value::{Addr, Value},
};

pub mod error;
pub mod scopes;

// program          => declaration* EOF ;
//
// declaration      => funDecl | varDecl | statement ;
// statement        => exprStmt
//                  | ifStmt;
//                  | printStmt
//                  | returnStmt
//                  | whileStmt
//                  | forStmt
//                  | block ;
// block            => "{" declaration* "}" ;
//
// funDecl          => "fun" function ;
// function         => IDENTIFIER "(" parameters? ")" block ;
// parameters       => IDENTIFIER ( "," IDENTIFIER )* ;
//
// varDecl          => "var" IDENTIFIER ( "=" expression )? ";" ;
// exprStmt         => expression ";" ;
// printStmt        => "print" expression ";" ;
// returnStmt       => "return" expression? ";" ;
// whileStmt        => "while" "(" expression ")" statement ;
// forStmt          => "for" "(" ( varDecl | exprStmt | ";" )
//                  expression? ";"
//                  expression? ")" statement ;
// ifStmt           => "if" "(" expression ")" statement
//                  ( "else" statement )? ;
//
// expression       => assignment ;
// assignment       => IDENTIFIER "=" assignment | logicOr ;
// logicOr          => logicAnd ( "or" logicAnd )*
// logicAnd         => equality ( "and" equality )*
// equality         => comparison ( ("!=" | "==") comparison )* ;
// comparison       => term ( (">" | ">=" | "<" | "<=") term )* ;
// term             => factor ( ("-" | "+") factor )* ;
// factor           => unary ( ("/" | "*") unary )* ;
// unary            => ("!" | "-") unary
//                  | call ;
// call             => primary ( "(" arguments? ")" )* ;
// arguments        => expression ( "," expression )* ;
//
// primary          => NUMBER | STRING
//                  | "true" | "false" | "nil"
//                  | "(" expression ")"
//                  | IDENTIFIER ;

#[derive(Debug, Clone, Copy)]
enum OpBinding {
    Postfix(u8),
    Infix(u8, u8),
}

fn op_bp(tok: &TokenType) -> Option<OpBinding> {
    // postfix
    if let Some(bp) = postfix_bp(tok) {
        return Some(OpBinding::Postfix(bp));
    }
    // infix
    infix_bp(tok).map(|(l, r)| OpBinding::Infix(l, r))
}

fn prefix_bp(tok: &TokenType) -> Option<u8> {
    Some(match tok {
        // not sure what BP it should be
        TokenType::LeftParen => 0,
        TokenType::Minus => 15,
        TokenType::Bang => 15,
        _ => return None,
    })
}

fn postfix_bp(_tok: &TokenType) -> Option<u8> {
    None
}

fn infix_bp(tok: &TokenType) -> Option<(u8, u8)> {
    Some(match tok {
        TokenType::Equal => (2, 1),
        TokenType::Or => (3, 4),
        TokenType::And => (5, 6),
        TokenType::EqualEqual | TokenType::BangEqual => (7, 8),
        TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual => {
            (9, 10)
        }
        TokenType::Plus | TokenType::Minus => (11, 12),
        TokenType::Star | TokenType::Slash => (13, 14),
        TokenType::Dot => (17, 18),
        TokenType::LeftParen => (19, 20),
        _ => return None,
    })
}

/// Result of compiling a sub-expression.
///
/// `Value` means the result is already pushed on the stack. `Place` means
/// compilation produced an addressable location that hasn't been read or
/// written yet; the next step (read in `materialize` or assign in `store`)
/// emits the appropriate get/set op.
#[must_use = "you should forward or materialize the Handle"]
#[derive(Debug, Clone, Copy)]
enum Handle {
    Value,
    Place(Place),
}

impl Handle {
    fn global(addr: Addr, line: u32) -> Self {
        Self::Place(Place::Global { addr, line })
    }

    fn local(slot: Slot, line: u32) -> Self {
        Self::Place(Place::Local { slot, line })
    }

    fn upvalue(slot: Slot, line: u32) -> Self {
        Self::Place(Place::Upvalue { slot, line })
    }
}

#[derive(Debug, Clone, Copy)]
enum Place {
    Global { addr: Addr, line: u32 },
    Local { slot: Slot, line: u32 },
    Upvalue { slot: Slot, line: u32 },
    Property { addr: Addr, line: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionKind {
    Script,
    Function,
    Method,
    Initializer,
}

impl FunctionKind {
    /// Method bodies bind the receiver to slot 0 under the name `this`; other
    /// functions leave slot 0 an unnameable callee placeholder.
    fn binds_this(self) -> bool {
        matches!(self, FunctionKind::Method | FunctionKind::Initializer)
    }
}

/// One enclosing class, tracked so `this`/`super` legality is structural: a
/// non-empty stack means "inside a class", and `has_superclass` gates `super`.
struct ClassScope {
    has_superclass: bool,
}

/// How a closure captures one upvalue: either a local of the immediately
/// enclosing function (`is_local`) or an upvalue the enclosing function itself
/// captured. Emitted as the `(is_local, index)` tail after `OP_CLOSURE`.
#[derive(Debug, Clone, Copy)]
struct UpvalueDesc {
    is_local: bool,
    index: u8,
}

/// Per-function compile state. The compiler holds a stack of these; the script
/// is the bottom target, and each `fun` pushes a fresh one whose chunk is
/// materialized into a `LoxFunction` when its body ends.
struct Target {
    chunk: Chunk,
    scopes: Scopes,
    kind: FunctionKind,
    upvalues: Vec<UpvalueDesc>,
}

impl Target {
    fn script() -> Self {
        Self::of(FunctionKind::Script)
    }

    fn of(kind: FunctionKind) -> Self {
        Self {
            chunk: Chunk::default(),
            scopes: Scopes::default(),
            kind,
            upvalues: Vec::new(),
        }
    }
}

pub struct Compiler<'s, 't> {
    scanner: Peekable<Scanner<'s>>,
    reporter: Reporter<'s>,
    storage: &'t mut Storage,
    errored: bool,
    targets: Vec<Target>,
    classes: Vec<ClassScope>,
}

impl<'s, 't> Compiler<'s, 't> {
    pub fn new(scanner: Scanner<'s>, reporter: Reporter<'s>, storage: &'t mut Storage) -> Self {
        Self {
            scanner: scanner.peekable(),
            reporter,
            storage,
            errored: false,
            targets: vec![Target::script()],
            classes: Vec::new(),
        }
    }

    fn target(&self) -> &Target {
        self.targets.last().expect("compiler always has a target")
    }

    fn target_mut(&mut self) -> &mut Target {
        self.targets
            .last_mut()
            .expect("compiler always has a target")
    }

    /// True only at the script's top level. A function body never counts as
    /// global, so its top-level `var`s become locals — even at scope depth 0.
    fn at_global_scope(&self) -> bool {
        let target = self.target();
        target.kind == FunctionKind::Script && target.scopes.is_global()
    }

    pub fn compile(&mut self) -> Result<Chunk, anyhow::Error> {
        while self.peek()?.is_some() {
            if let Err(e) = self.declaration() {
                self.errored = true;
                self.report_err(e);
                self.synchronize()?;
            }
        }
        self.end();

        match self.errored {
            true => bail!("Compilation failed"),
            false => Ok(std::mem::take(&mut self.target_mut().chunk)),
        }
    }

    pub fn end(&mut self) {
        // The script returns implicitly: push the (discarded) return value the
        // new `Ret` semantics expect, then halt.
        self.emit_op(OpCode::Nil);
        self.emit_op(OpCode::Ret);
    }

    pub fn report_err(&self, err: CompileError) {
        match err {
            CompileError::Lexing(e) => self.reporter.report(&e),
            CompileError::Parsing(e) => self.reporter.report(&e),
            CompileError::Other(e) => self.reporter.report_unspanned(&e),
        }
    }

    fn declaration(&mut self) -> Result<(), CompileError> {
        let Some(tok) = self.peek()? else {
            return Ok(());
        };

        match tok.ty() {
            TokenType::Var => self.var_decl(),
            TokenType::Fun => self.fun_decl(),
            TokenType::Class => self.class_decl(),
            _ => self.statement(),
        }
    }

    fn class_decl(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::Class)?;
        let ident = self.consume_with(|t| matches!(t, TokenType::Identifier(_)), "class name")?;
        let name = self.storage.intern(&ident.as_str());
        let line = ident.line();
        let addr = self.ident_constant(name)?;

        // Bind the class to its variable exactly like `var`: a global is popped
        // into the globals table; a local stays in its slot.
        let global = self.at_global_scope();
        if !global {
            self.target_mut()
                .scopes
                .declare(name)
                .context("declaring local class")?;
        }
        self.emit_op_and_line(line, OpCode::Class(addr));
        if global {
            self.emit_op_and_line(line, OpCode::DefGlobal(addr));
        }

        self.classes.push(ClassScope {
            has_superclass: false,
        });
        let has_superclass = self.superclass_clause(name, line)?;

        // Reload the class so each OP_METHOD has it on the stack to bind against.
        self.load_variable(name, line, ident.span)?;
        self.consume(TokenType::LeftBrace)
            .context("Expect '{' before class body.")?;
        while let Some(tok) = self.peek()?
            && tok.ty != TokenType::RightBrace
        {
            self.method()?;
        }
        self.consume(TokenType::RightBrace)
            .context("Expect '}' after class body.")?;
        self.emit_op_and_line(line, OpCode::Pop); // the reloaded class

        if has_superclass {
            // Close the scope holding the synthetic `super` local (and its
            // upvalue, if a method captured it).
            let captured = self.target_mut().scopes.exit();
            self.emit_scope_exit(captured);
        }
        self.classes.pop();
        Ok(())
    }

    /// Compile a `< Superclass` clause: load the superclass, splice its methods
    /// into the subclass via `OP_INHERIT`, and bind it to a synthetic `super`
    /// local that methods capture. Returns whether a superclass was present.
    fn superclass_clause(&mut self, name: Spur, line: u32) -> Result<bool, CompileError> {
        if self.advance_if(TokenType::Less)?.is_none() {
            return Ok(false);
        }
        let super_ident =
            self.consume_with(|t| matches!(t, TokenType::Identifier(_)), "superclass name")?;
        let super_name = self.storage.intern(&super_ident.as_str());
        if super_name == name {
            return Err(
                ParsingError::custom(&super_ident, "A class can't inherit from itself.").into(),
            );
        }

        self.load_variable(super_name, super_ident.line(), super_ident.span)?;
        // The superclass lives in a fresh scope as the synthetic `super` local,
        // so method bodies capture it as an upvalue.
        self.target_mut().scopes.enter();
        let super_spur = self.storage.intern("super");
        self.target_mut()
            .scopes
            .declare(super_spur)
            .context("declaring super")?;

        self.load_variable(name, line, super_ident.span)?;
        self.emit_op_and_line(line, OpCode::Inherit);
        self.classes
            .last_mut()
            .expect("inside a class")
            .has_superclass = true;
        Ok(true)
    }

    fn method(&mut self) -> Result<(), CompileError> {
        let ident = self.consume_with(|t| matches!(t, TokenType::Identifier(_)), "method name")?;
        let name = self.storage.intern(&ident.as_str());
        let addr = self.ident_constant(name)?;
        let line = ident.line();

        let kind = if name == self.storage.intern("init") {
            FunctionKind::Initializer
        } else {
            FunctionKind::Method
        };
        self.function(kind, name, line)?;
        self.emit_op_and_line(line, OpCode::Method(addr));
        Ok(())
    }

    fn var_decl(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::Var)?;
        let ident = self.consume_with(
            |t| matches!(t, TokenType::Identifier(_)),
            "variable identifier",
        )?;

        if self.at_global_scope() {
            self.global_var_decl(ident)
        } else {
            self.local_var_decl(ident)
        }
    }

    fn global_var_decl(&mut self, ident: Token) -> Result<(), CompileError> {
        let name = self.storage.intern(&ident.as_str());
        let addr = self.ident_constant(name)?;
        self.var_initializer(&ident)?;
        let semi = self.consume(TokenType::Semicolon)?;
        self.emit_op_and_line(semi.line(), OpCode::DefGlobal(addr));
        Ok(())
    }

    fn local_var_decl(&mut self, ident: Token) -> Result<(), CompileError> {
        let name = self.storage.intern(&ident.as_str());
        self.var_initializer(&ident)?;
        self.consume(TokenType::Semicolon)?;

        // Declare AFTER the initializer is compiled so `var a = a + 1;`
        // refers to the previously-bound `a` (rather than itself). This is
        // a deliberate deviation from Lox spec — Rust-style shadowing.
        self.target_mut()
            .scopes
            .declare(name)
            .context("declaring local")?;
        Ok(())
    }

    fn var_initializer(&mut self, ident: &Token) -> Result<(), CompileError> {
        match self.advance_if(TokenType::Equal)? {
            Some(_) => self.expression(),
            None => {
                self.emit_op_and_line(ident.line(), OpCode::Nil);
                Ok(())
            }
        }
    }

    fn fun_decl(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::Fun)?;
        let ident =
            self.consume_with(|t| matches!(t, TokenType::Identifier(_)), "function name")?;
        let name = self.storage.intern(&ident.as_str());
        let line = ident.line();

        if self.at_global_scope() {
            let addr = self.ident_constant(name)?;
            self.function(FunctionKind::Function, name, line)?;
            self.emit_op_and_line(line, OpCode::DefGlobal(addr));
        } else {
            // Declare the name BEFORE the body so the function can recurse — a
            // function value is never observed half-initialized, unlike `var`.
            self.target_mut()
                .scopes
                .declare(name)
                .context("declaring local function")?;
            self.function(FunctionKind::Function, name, line)?;
        }
        Ok(())
    }

    /// Compile a function body into its own `Target`, materialize it as a
    /// `LoxFunction`, and emit an `OP_CLOSURE` for it in the enclosing chunk.
    fn function(&mut self, kind: FunctionKind, name: Spur, line: u32) -> Result<(), CompileError> {
        self.targets.push(Target::of(kind));
        // Slot 0 is the callee's runtime home (`base + 0`): named `this` in a
        // method so the receiver resolves there, otherwise an unnameable
        // placeholder the lexer can never produce.
        let slot0 = self
            .storage
            .intern(if kind.binds_this() { "this" } else { "" });
        self.target_mut()
            .scopes
            .declare(slot0)
            .expect("slot 0 fits in a fresh scope");

        self.consume(TokenType::LeftParen)
            .context("Expect '(' after function name.")?;
        let arity = self.parameters()?;
        self.consume(TokenType::LeftBrace)
            .context("Expect '{' before function body.")?;
        self.block()?;

        // Implicit return. A trailing explicit return just makes this dead code.
        self.emit_return(line);

        let Target {
            chunk, upvalues, ..
        } = self.targets.pop().expect("function target");
        let func = LoxFunction::new(name, arity, upvalues.len() as u16, chunk);
        let obj = self.storage.add_obj(Box::new(func));
        let addr = self.add_constant(Value::object(obj))?;
        // Wrap the function in a closure at runtime; the trailing bytes tell the
        // VM where each captured upvalue comes from.
        self.emit_op_and_line(line, OpCode::Closure(addr));
        for uv in &upvalues {
            self.emit_upvalue_bytes(line, uv.is_local, uv.index);
        }
        Ok(())
    }

    fn emit_upvalue_bytes(&mut self, line: u32, is_local: bool, index: u8) {
        let chunk = &mut self.target_mut().chunk;
        chunk.write_byte_with_line(line, is_local as u8);
        chunk.write_byte_with_line(line, index);
    }

    fn parameters(&mut self) -> Result<u8, CompileError> {
        let mut arity = 0u8;
        let has_params = matches!(self.peek()?, Some(tok) if tok.ty != TokenType::RightParen);
        if has_params {
            loop {
                if arity == u8::MAX {
                    let span = self.peek()?.map(|t| t.span).unwrap_or_default();
                    return Err(
                        ParsingError::custom(span, "Can't have more than 255 parameters.").into(),
                    );
                }
                let param =
                    self.consume_with(|t| matches!(t, TokenType::Identifier(_)), "parameter name")?;
                let pname = self.storage.intern(&param.as_str());
                self.target_mut()
                    .scopes
                    .declare(pname)
                    .context("declaring parameter")?;
                arity += 1;
                if self.advance_if(TokenType::Comma)?.is_none() {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen)
            .context("Expect ')' after parameters.")?;
        Ok(arity)
    }

    fn return_stmt(&mut self) -> Result<(), CompileError> {
        let ret = self
            .consume(TokenType::Return)
            .expect("matched token before entering this branch");
        if self.target().kind == FunctionKind::Script {
            return Err(ParsingError::custom(&ret, "Can't return from top-level code.").into());
        }

        if self.advance_if(TokenType::Semicolon)?.is_some() {
            self.emit_return(ret.line());
        } else {
            if self.target().kind == FunctionKind::Initializer {
                return Err(ParsingError::custom(
                    &ret,
                    "Can't return a value from an initializer.",
                )
                .into());
            }
            self.expression()?;
            let semi = self
                .consume(TokenType::Semicolon)
                .context("Expect ';' after return value.")?;
            self.emit_op_and_line(semi.line(), OpCode::Ret);
        }
        Ok(())
    }

    /// Emit the implicit return value (`this` in an initializer, else `nil`) and
    /// the return op.
    fn emit_return(&mut self, line: u32) {
        if self.target().kind == FunctionKind::Initializer {
            self.emit_op_and_line(line, OpCode::GetLocal(0));
        } else {
            self.emit_op_and_line(line, OpCode::Nil);
        }
        self.emit_op_and_line(line, OpCode::Ret);
    }

    fn statement(&mut self) -> Result<(), CompileError> {
        let Some(tok) = self.peek()? else {
            return Ok(());
        };
        match tok.ty() {
            TokenType::Print => self.print_stmt(),
            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::For => self.for_stmt(),
            TokenType::Return => self.return_stmt(),
            TokenType::LeftBrace => self.block_stmt(),
            _ => self.expression_stmt(),
        }
    }

    fn if_stmt(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::If)
            .expect("matched token before entering this branch");
        self.consume(TokenType::LeftParen)
            .context("expect '(' after 'if'.")?;
        self.expression()?;
        let tok = self
            .consume(TokenType::RightParen)
            .context("expect ')' after condition.")?;

        // `JmpIfFalsePop` discards the condition on both paths, so no separate
        // `Pop` is needed around either branch.
        let then_jmp = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalsePop(0));
        self.statement()?;

        let else_jmp = self.emit_jmp(OpCode::Jmp(0));
        self.patch_jmp(then_jmp)?;

        if self.advance_if(TokenType::Else)?.is_some() {
            self.statement()?;
        }
        self.patch_jmp(else_jmp)?;

        Ok(())
    }

    fn while_stmt(&mut self) -> Result<(), CompileError> {
        let loop_start = self.target().chunk.current();
        self.consume(TokenType::While)
            .expect("matched token before entering this branch");
        self.consume(TokenType::LeftParen)
            .context("Expect '(' after 'while'.")?;
        self.expression()?;
        let tok = self
            .consume(TokenType::RightParen)
            .context("Expect ')' after 'condition'.")?;

        let exit_jmp = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalsePop(0));
        self.statement()?;
        self.emit_loop(loop_start)?;

        self.patch_jmp(exit_jmp)?;
        Ok(())
    }

    fn for_stmt(&mut self) -> Result<(), CompileError> {
        let mut this = self.begin_scope();
        this.consume(TokenType::For)
            .expect("matched token before entering this branch");
        this.consume(TokenType::LeftParen)
            .context("Expect '(' after 'for'.")?;

        match this
            .peek()?
            .context("Unexpected EOF in for initializer.")?
            .ty()
        {
            TokenType::Semicolon => {
                this.consume(TokenType::Semicolon)?;
            }
            TokenType::Var => this.var_decl()?,
            _ => this.expression_stmt()?,
        };

        let mut loop_start = this.target().chunk.current();
        let exit_jmp = if this.advance_if(TokenType::Semicolon)?.is_none() {
            this.expression()?;
            let tok = this
                .consume(TokenType::Semicolon)
                .context("Expect ';' after a loop condition.")?;
            let exit_jmp = this.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalsePop(0));
            Some(exit_jmp)
        } else {
            None
        };

        if this.advance_if(TokenType::RightParen)?.is_none() {
            let body_jmp = this.emit_jmp(OpCode::Jmp(0));
            let inc_start = this.target().chunk.current();
            this.expression()?;
            this.emit_pops(1);
            this.consume(TokenType::RightParen)
                .context("Expect ')' after for clauses.")?;

            this.emit_loop(loop_start)?;
            loop_start = inc_start;
            this.patch_jmp(body_jmp)?;
        }

        this.statement()?;
        this.emit_loop(loop_start)?;

        if let Some(exit_jmp) = exit_jmp {
            this.patch_jmp(exit_jmp)?;
        }
        Ok(())
    }

    fn print_stmt(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::Print)
            .expect("matched token before entering this branch");
        self.expression()?;
        let tok = self
            .consume(TokenType::Semicolon)
            .context("Missing semicolon")?;
        self.emit_op_and_line(tok.line(), OpCode::Print);
        Ok(())
    }

    fn block_stmt(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::LeftBrace)
            .expect("matched left brace before entering this branch");
        let mut this = self.begin_scope();
        this.block()?;
        Ok(())
    }

    fn block(&mut self) -> Result<(), CompileError> {
        while let Some(tok) = self.peek()?
            && tok.ty != TokenType::RightBrace
        {
            self.declaration()?;
        }
        self.consume(TokenType::RightBrace)?;
        Ok(())
    }

    fn expression_stmt(&mut self) -> Result<(), CompileError> {
        self.expression()?;
        self.consume(TokenType::Semicolon)?;
        self.emit_pops(1);
        Ok(())
    }

    fn expression(&mut self) -> Result<(), CompileError> {
        let handle = self.parse_bp(0)?;
        self.materialize(handle);
        Ok(())
    }

    fn parse_bp(&mut self, min_bp: u8) -> Result<Handle, CompileError> {
        let lhs = self
            .advance()?
            .ok_or(ParsingError::expected(Span::default(), "token", "EOF"))?;
        let mut handle = self.parse_prefix(lhs)?;

        loop {
            let op = match self.peek()? {
                None => break,
                Some(tok) => tok,
            };

            match op_bp(&op.ty) {
                Some(OpBinding::Postfix(l_bp)) if l_bp >= min_bp => {
                    let op = self.advance().unwrap().unwrap();
                    handle = self.parse_postfix(op, handle)?;
                }
                Some(OpBinding::Infix(l_bp, _r_bp)) if l_bp >= min_bp => {
                    let op = self.advance().unwrap().unwrap();
                    handle = self.parse_infix(op, handle)?;
                }
                _ => break,
            };
        }
        Ok(handle)
    }

    fn parse_prefix(&mut self, tok: Token) -> Result<Handle, CompileError> {
        match tok.ty() {
            TokenType::LeftParen => self.grouping(tok),
            TokenType::Minus | TokenType::Bang => self.unary(tok),
            TokenType::Number(_) => self.number(tok),
            TokenType::String(_) => self.string(tok),
            TokenType::True | TokenType::False | TokenType::Nil => self.literal(tok),
            TokenType::Identifier(_) => self.named_variable(tok),
            TokenType::This => self.this_expr(tok),
            TokenType::Super => self.super_expr(tok),
            _ => Err(ParsingError::expected(&tok, "expression", &tok).into()),
        }
    }

    fn super_expr(&mut self, tok: Token) -> Result<Handle, CompileError> {
        match self.classes.last() {
            None => {
                return Err(
                    ParsingError::custom(&tok, "Can't use 'super' outside of a class.").into(),
                );
            }
            Some(class) if !class.has_superclass => {
                return Err(ParsingError::custom(
                    &tok,
                    "Can't use 'super' in a class with no superclass.",
                )
                .into());
            }
            Some(_) => {}
        }

        self.consume(TokenType::Dot)
            .context("Expect '.' after 'super'.")?;
        let method = self.consume_with(
            |t| matches!(t, TokenType::Identifier(_)),
            "superclass method name",
        )?;
        let name = self.storage.intern(&method.as_str());
        let addr = self.ident_constant(name)?;
        let (line, span) = (method.line(), method.span);

        let this_spur = self.storage.intern("this");
        let super_spur = self.storage.intern("super");
        // `super.m(args)` fuses the lookup and the call; a bare `super.m` binds.
        // The receiver (`this`) goes down first, then args, then the superclass.
        self.load_variable(this_spur, line, span)?;
        if matches!(self.peek()?, Some(tok) if tok.ty == TokenType::LeftParen) {
            self.advance()?; // consume '('
            let arg_count = self.argument_list()?;
            self.load_variable(super_spur, line, span)?;
            self.emit_op_and_line(line, OpCode::SuperInvoke(addr, arg_count));
        } else {
            self.load_variable(super_spur, line, span)?;
            self.emit_op_and_line(line, OpCode::GetSuper(addr));
        }
        Ok(Handle::Value)
    }

    fn this_expr(&mut self, tok: Token) -> Result<Handle, CompileError> {
        if self.classes.is_empty() {
            return Err(ParsingError::custom(&tok, "Can't use 'this' outside of a class.").into());
        }
        // `this` is slot 0 of a method (or an upvalue of a nested function), and
        // is never assignable — materialize it straight to a value.
        let handle = self.named_variable(tok)?;
        self.materialize(handle);
        Ok(Handle::Value)
    }

    fn parse_infix(&mut self, tok: Token, lhs: Handle) -> Result<Handle, CompileError> {
        match tok.ty() {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::BangEqual
            | TokenType::EqualEqual
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => self.binary(tok, lhs),
            TokenType::Equal => self.assignment(tok, lhs),
            TokenType::And => self.and(tok, lhs),
            TokenType::Or => self.or(tok, lhs),
            TokenType::LeftParen => self.call(tok, lhs),
            TokenType::Dot => self.dot(tok, lhs),
            _ => Err(ParsingError::expected(&tok, "expression", &tok).into()),
        }
    }

    #[expect(unused)]
    fn parse_postfix(&mut self, tok: Token, lhs: Handle) -> Result<Handle, CompileError> {
        unimplemented!("no postfix ops atm");
    }

    fn grouping(&mut self, _tok: Token) -> Result<Handle, CompileError> {
        self.expression()?;
        self.consume(TokenType::RightParen)?;
        Ok(Handle::Value)
    }

    fn unary(&mut self, op: Token) -> Result<Handle, CompileError> {
        let r_bp = prefix_bp(op.ty()).expect("expected infix op token");
        let operand = self.parse_bp(r_bp)?;
        self.materialize(operand);

        let line = op.line();
        match op.ty() {
            TokenType::Minus => self.emit_op_and_line(line, OpCode::Neg),
            TokenType::Bang => self.emit_op_and_line(line, OpCode::Not),
            _ => panic!("expected minus token as prefix"),
        };

        Ok(Handle::Value)
    }

    fn number(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let Token {
            ty: TokenType::Number(num),
            span,
        } = tok
        else {
            unreachable!("expected number token");
        };
        self.emit_constant_and_line(span.line_start, Value::number(num))?;
        Ok(Handle::Value)
    }

    fn string(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let Token {
            ty: TokenType::String(s),
            span,
        } = tok
        else {
            unreachable!("expected string token");
        };
        let key = self.storage.intern(&s);
        self.emit_constant_and_line(span.line_start, Value::symbol(key))?;
        Ok(Handle::Value)
    }

    fn literal(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let line = tok.line();
        match tok.ty() {
            TokenType::True => self.emit_op_and_line(line, OpCode::True),
            TokenType::False => self.emit_op_and_line(line, OpCode::False),
            TokenType::Nil => self.emit_op_and_line(line, OpCode::Nil),
            _ => panic!("expected literal tokens"),
        }
        Ok(Handle::Value)
    }

    fn named_variable(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let name = self.storage.intern(&tok.as_str());
        self.resolve_named(name, tok.line(), tok.span)
    }

    /// Resolve a name to a read/write `Place`: local, then an upvalue captured
    /// from an enclosing function, then global.
    fn resolve_named(&mut self, name: Spur, line: u32, span: Span) -> Result<Handle, CompileError> {
        if let Some(slot) = self.target().scopes.resolve(name) {
            return Ok(Handle::local(slot, line));
        }
        if let Some(slot) = self.resolve_upvalue(self.targets.len() - 1, name, span)? {
            return Ok(Handle::upvalue(slot, line));
        }
        let addr = self.ident_constant(name)?;
        Ok(Handle::global(addr, line))
    }

    /// Emit a read of `name` (used to reload a class for method binding).
    fn load_variable(&mut self, name: Spur, line: u32, span: Span) -> Result<(), CompileError> {
        let handle = self.resolve_named(name, line, span)?;
        self.materialize(handle);
        Ok(())
    }

    /// Resolve `name` to an upvalue of `target_idx` by walking enclosing
    /// functions: capture it as a local of the immediately enclosing function,
    /// or recursively as one of that function's own upvalues. Returns the
    /// upvalue's index within `target_idx`'s closure, or `None` for a global.
    fn resolve_upvalue(
        &mut self,
        target_idx: usize,
        name: Spur,
        span: Span,
    ) -> Result<Option<u8>, CompileError> {
        if target_idx == 0 {
            return Ok(None); // the script captures nothing
        }
        let enclosing = target_idx - 1;

        if let Some(local) = self.targets[enclosing].scopes.resolve(name) {
            self.targets[enclosing].scopes.mark_captured(local);
            return self.add_upvalue(target_idx, local, true, span).map(Some);
        }
        if let Some(upvalue) = self.resolve_upvalue(enclosing, name, span)? {
            return self.add_upvalue(target_idx, upvalue, false, span).map(Some);
        }
        Ok(None)
    }

    fn add_upvalue(
        &mut self,
        target_idx: usize,
        index: u8,
        is_local: bool,
        span: Span,
    ) -> Result<u8, CompileError> {
        let upvalues = &self.targets[target_idx].upvalues;
        if let Some(existing) = upvalues
            .iter()
            .position(|u| u.index == index && u.is_local == is_local)
        {
            return Ok(existing as u8);
        }
        // The upvalue index is a `u8` operand, so 256 (indices 0..=255) is the cap.
        if upvalues.len() > u8::MAX as usize {
            return Err(
                ParsingError::custom(span, "Too many closure variables in function.").into(),
            );
        }
        let slot = upvalues.len() as u8;
        self.targets[target_idx]
            .upvalues
            .push(UpvalueDesc { is_local, index });
        Ok(slot)
    }

    fn binary(&mut self, op: Token, lhs: Handle) -> Result<Handle, CompileError> {
        self.materialize(lhs);
        let (_l_bp, r_bp) = infix_bp(op.ty()).expect("expected infix op token");
        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);

        let line = op.line();
        #[rustfmt::skip]
        match op.ty() {
            TokenType::Plus => self.emit_op_and_line(line, OpCode::Add),
            TokenType::Minus => self.emit_op_and_line(line, OpCode::Sub),
            TokenType::Star => self.emit_op_and_line(line, OpCode::Mul),
            TokenType::Slash => self.emit_op_and_line(line, OpCode::Div),
            TokenType::BangEqual => {
                self.emit_op_and_line(line, OpCode::Equal);
                self.emit_op_and_line(line, OpCode::Not);
            }
            TokenType::EqualEqual => self.emit_op_and_line(line, OpCode::Equal),
            TokenType::Greater => self.emit_op_and_line(line, OpCode::Greater),
            TokenType::GreaterEqual => {
                self.emit_op_and_line(line, OpCode::Less);
                self.emit_op_and_line(line, OpCode::Not);
            }
            TokenType::Less => self.emit_op_and_line(line, OpCode::Less),
            TokenType::LessEqual => {
                self.emit_op_and_line(line, OpCode::Greater);
                self.emit_op_and_line(line, OpCode::Not);
            }
            _ => panic!("unexpected binary token: {op}"),
        };
        Ok(Handle::Value)
    }

    fn and(&mut self, tok: Token, lhs: Handle) -> Result<Handle, CompileError> {
        let (_l_bp, r_bp) = infix_bp(tok.ty()).expect("expected infix op token");
        self.materialize(lhs);

        let short_circuit = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalse(0));
        self.emit_op_and_line(tok.line(), OpCode::Pop);

        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);
        self.patch_jmp(short_circuit)?;

        Ok(Handle::Value)
    }

    fn or(&mut self, tok: Token, lhs: Handle) -> Result<Handle, CompileError> {
        let (_l_bp, r_bp) = infix_bp(tok.ty()).expect("expected infix op token");
        self.materialize(lhs);

        // Mirror `and` exactly, just with the opposite test: a single
        // conditional jump (`JmpIfTrue`) short-circuits to the result, so `or`
        // is no slower than `and` (two jumps + a pop before).
        let short_circuit = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfTrue(0));
        self.emit_op_and_line(tok.line(), OpCode::Pop);

        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);
        self.patch_jmp(short_circuit)?;

        Ok(Handle::Value)
    }

    fn assignment(&mut self, equal: Token, lhs: Handle) -> Result<Handle, CompileError> {
        let Handle::Place(place) = lhs else {
            return Err(ParsingError::expected(&equal, "lvalue", "rvalue").into());
        };
        let (_, r_bp) = infix_bp(equal.ty()).expect("=");
        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);
        self.store(place);
        Ok(Handle::Value)
    }

    fn call(&mut self, paren: Token, callee: Handle) -> Result<Handle, CompileError> {
        self.materialize(callee);
        let arg_count = self.argument_list()?;
        self.emit_op_and_line(paren.line(), OpCode::Call(arg_count));
        Ok(Handle::Value)
    }

    fn argument_list(&mut self) -> Result<u8, CompileError> {
        let mut arg_count = 0u8;
        let has_args = matches!(self.peek()?, Some(tok) if tok.ty != TokenType::RightParen);
        if has_args {
            loop {
                if arg_count == u8::MAX {
                    let span = self.peek()?.map(|t| t.span).unwrap_or_default();
                    return Err(
                        ParsingError::custom(span, "Can't have more than 255 arguments.").into(),
                    );
                }
                self.expression()?;
                arg_count += 1;
                if self.advance_if(TokenType::Comma)?.is_none() {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen)
            .context("Expect ')' after arguments.")?;
        Ok(arg_count)
    }

    fn dot(&mut self, _dot: Token, instance: Handle) -> Result<Handle, CompileError> {
        self.materialize(instance);
        let prop = self.consume_with(
            |t| matches!(t, TokenType::Identifier(_)),
            "property name after '.'",
        )?;
        let name = self.storage.intern(&prop.as_str());
        let addr = self.ident_constant(name)?;
        let line = prop.line();

        // `recv.name(args)` fuses into a single OP_INVOKE — no bound-method
        // allocation. A bare `recv.name` stays a Place so `=` becomes
        // SetProperty and a read becomes GetProperty, reusing the lvalue path.
        if matches!(self.peek()?, Some(tok) if tok.ty == TokenType::LeftParen) {
            self.advance()?; // consume '('
            let arg_count = self.argument_list()?;
            self.emit_op_and_line(line, OpCode::Invoke(addr, arg_count));
            return Ok(Handle::Value);
        }
        Ok(Handle::Place(Place::Property { addr, line }))
    }

    fn materialize(&mut self, handle: Handle) {
        match handle {
            Handle::Value => {}
            Handle::Place(Place::Global { addr, line }) => {
                self.emit_op_and_line(line, OpCode::GetGlobal(addr));
            }
            Handle::Place(Place::Local { slot, line }) => {
                self.emit_op_and_line(line, OpCode::GetLocal(slot));
            }
            Handle::Place(Place::Upvalue { slot, line }) => {
                self.emit_op_and_line(line, OpCode::GetUpvalue(slot));
            }
            Handle::Place(Place::Property { addr, line }) => {
                self.emit_op_and_line(line, OpCode::GetProperty(addr));
            }
        }
    }

    fn store(&mut self, place: Place) {
        match place {
            Place::Global { addr, line } => {
                self.emit_op_and_line(line, OpCode::SetGlobal(addr));
            }
            Place::Local { slot, line } => {
                self.emit_op_and_line(line, OpCode::SetLocal(slot));
            }
            Place::Upvalue { slot, line } => {
                self.emit_op_and_line(line, OpCode::SetUpvalue(slot));
            }
            Place::Property { addr, line } => {
                self.emit_op_and_line(line, OpCode::SetProperty(addr));
            }
        }
    }

    fn ident_constant(&mut self, name: Spur) -> Result<Addr, CompileError> {
        self.add_constant(Value::symbol(name))
    }

    #[must_use]
    fn begin_scope<'c>(
        &'c mut self,
    ) -> ScopeGuard<&'c mut Compiler<'s, 't>, impl FnOnce(&'c mut Compiler<'s, 't>)> {
        self.target_mut().scopes.enter();
        scopeguard::guard(self, |this| {
            let captured = this.target_mut().scopes.exit();
            this.emit_scope_exit(captured);
        })
    }

    /// On scope exit, pop each local — but a captured local needs `CloseUpvalue`
    /// to hoist it into its upvalue first. The common all-uncaptured case stays
    /// a single `PopN`.
    fn emit_scope_exit(&mut self, captured: Vec<bool>) {
        if captured.iter().all(|&c| !c) {
            self.emit_pops(captured.len());
        } else {
            for cap in captured {
                if cap {
                    self.emit_op(OpCode::CloseUpvalue);
                } else {
                    self.emit_op(OpCode::Pop);
                }
            }
        }
    }

    fn emit_pops(&mut self, mut count: usize) {
        // A scope can hold up to 256 locals (`MAX_LOCALS`), one more than a
        // single `PopN`'s `u8` operand encodes, so chunk the count.
        while count > u8::MAX as usize {
            self.emit_op(OpCode::PopN(u8::MAX));
            count -= u8::MAX as usize;
        }
        match count {
            0 => {}
            1 => self.emit_op(OpCode::Pop),
            n => self.emit_op(OpCode::PopN(n as u8)),
        }
    }

    fn emit_op(&mut self, op: OpCode) {
        self.target_mut().chunk.write(op);
    }

    fn emit_op_and_line(&mut self, line: u32, op: OpCode) {
        self.target_mut().chunk.write_with_line(line, op);
    }

    fn add_constant(&mut self, value: Value) -> Result<Addr, CompileError> {
        if let Some(addr) = self
            .target()
            .chunk
            .constants
            .iter()
            .rposition(|v| v == &value)
        {
            return Ok(addr as Addr);
        }
        // `Addr` is a `u8`, so 256 (indices 0..=255) is the cap. Report rather
        // than panic so a program with too many constants is a clean error.
        if self.target().chunk.constants.len() > u8::MAX as usize {
            return Err(
                ParsingError::custom(Span::default(), "Too many constants in one chunk.").into(),
            );
        }
        Ok(self.target_mut().chunk.add_constant(value))
    }

    fn emit_constant_and_line(&mut self, line: u32, value: Value) -> Result<Addr, CompileError> {
        let addr = self.add_constant(value)?;
        self.target_mut()
            .chunk
            .write_with_line(line, OpCode::Constant(addr));
        Ok(addr)
    }

    fn emit_jmp_and_line(&mut self, line: u32, op: OpCode) -> u64 {
        self.emit_op_and_line(line, op);
        self.target().chunk.current()
    }

    fn emit_jmp(&mut self, op: OpCode) -> u64 {
        self.emit_op(op);
        self.target().chunk.current()
    }

    fn emit_loop(&mut self, loop_start: u64) -> Result<(), CompileError> {
        let offset = self.target().chunk.current() - loop_start + 3; // sizeof(OP_LOOP) = 3
        if offset > u16::MAX as u64 {
            return Err(ParsingError::custom(Span::default(), "Loop body too large.").into());
        }
        self.emit_op(OpCode::Loop(offset as u16));
        Ok(())
    }

    fn patch_jmp(&mut self, offset: u64) -> Result<(), CompileError> {
        let jmp = self.target().chunk.current() - offset;
        if jmp > u16::MAX as u64 {
            return Err(
                ParsingError::custom(Span::default(), "Too much code to jump over.").into(),
            );
        }
        self.target_mut()
            .chunk
            .write_raw(offset - 2, &(jmp as u16).to_le_bytes());
        Ok(())
    }

    fn advance(&mut self) -> Result<Option<Token>, LexingError> {
        self.scanner.next().transpose()
    }

    fn peek(&mut self) -> Result<Option<&Token>, LexingError> {
        self.scanner
            .peek()
            .map(|res| res.as_ref().map_err(|err| err.clone()))
            .transpose()
    }

    fn advance_if(&mut self, pattern: TokenType) -> Result<Option<Token>, LexingError> {
        match self.peek()? {
            Some(tok) if pattern == tok.ty => self.advance(),
            Some(_) | None => Ok(None),
        }
    }

    fn consume(&mut self, pattern: TokenType) -> Result<Token, CompileError> {
        match self.advance()? {
            Some(tok) if pattern == tok.ty => Ok(tok),
            Some(tok) => Err(ParsingError::expected(&tok, pattern, &tok).into()),
            None => Err(ParsingError::expected(Span::default(), pattern, TokenType::Eof).into()),
        }
    }

    fn consume_with(
        &mut self,
        pattern_fn: impl FnOnce(&TokenType) -> bool,
        expected_item: impl Display,
    ) -> Result<Token, CompileError> {
        match self.advance()? {
            Some(tok) if pattern_fn(&tok.ty) => Ok(tok),
            Some(tok) => Err(ParsingError::expected(&tok, expected_item, &tok).into()),
            None => {
                Err(ParsingError::expected(Span::default(), expected_item, TokenType::Eof).into())
            }
        }
    }

    fn synchronize(&mut self) -> Result<(), LexingError> {
        // should only return errors in case of a lexing error
        while let Some(tok) = self.advance()? {
            match tok.ty() {
                TokenType::Semicolon => return Ok(()),
                _ => match self.peek()?.map(|t| t.ty()) {
                    Some(
                        TokenType::Class
                        | TokenType::For
                        | TokenType::Fun
                        | TokenType::If
                        | TokenType::Print
                        | TokenType::Return
                        | TokenType::Var
                        | TokenType::While,
                    ) => return Ok(()),
                    _ => continue,
                },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(src: &str) -> Chunk {
        let mut storage = Storage::new();
        Compiler::new(Scanner::new(src), Reporter::new(src), &mut storage)
            .compile()
            .unwrap_or_else(|_| panic!("failed to compile `{src}`"))
    }

    #[test]
    fn challenge() {
        compile("(-1 + 2) * 3 - -4;");
    }

    #[test]
    fn arithmetic() {
        compile("2 * 3 + 4;");
    }

    #[test]
    fn prefix() {
        compile("-2 * 3 + 4;");
    }

    #[test]
    fn grouping() {
        compile("2 * (3 + 4);");
    }

    #[test]
    fn logical() {
        compile("!(5 - 4 > 3 * 2 == !nil);");
    }

    #[test]
    fn string_literal() {
        compile("\"hello\";");
    }

    #[test]
    fn string_ops() {
        compile("\"a\" + \"b\" == \"ab\";");
    }

    #[test]
    fn var_declaration() {
        compile("var a = 1 + 2; print a;");
    }

    #[test]
    fn var_assignment() {
        compile("var a = 1 + 2; a = 0;");
    }

    #[test]
    fn block_empty() {
        compile("{}");
    }

    #[test]
    fn block_with_stmt() {
        compile("{ print 1; }");
    }

    #[test]
    fn block_nested() {
        compile("{ { print 1; } }");
    }

    #[test]
    fn block_with_local_var() {
        compile("{ var a = 1; print a; }");
    }

    #[test]
    fn local_shadow_same_scope() {
        compile("{ var a = 1; var a = 2; print a; }");
    }

    #[test]
    fn local_shadow_uses_previous() {
        compile("{ var a = 1; var a = a + 1; print a; }");
    }

    #[test]
    fn nested_block_shadow() {
        compile("{ var a = 1; { var a = a + 1; print a; } print a; }");
    }

    #[test]
    fn if_without_else() {
        compile("if (true) print 1;");
    }

    #[test]
    fn if_else() {
        compile("if (1 < 2) print 1; else print 2;");
    }

    #[test]
    fn logical_and() {
        compile("true and false;");
    }

    #[test]
    fn logical_or() {
        compile("nil or 1;");
    }

    #[test]
    fn logical_and_or_mixed() {
        compile("1 and 2 or 3 and nil;");
    }

    #[test]
    fn for_full() {
        compile("for (var i = 0; i < 3; i = i + 1) { print i; }");
    }

    #[test]
    fn for_no_initializer() {
        compile("var i = 0; for (; i < 3; i = i + 1) print i;");
    }

    #[test]
    fn for_no_condition() {
        compile("for (var i = 0;; i = i + 1) print i;");
    }

    #[test]
    fn for_no_increment() {
        compile("for (var i = 0; i < 3;) { print i; i = i + 1; }");
    }

    #[test]
    fn for_empty_clauses() {
        compile("for (;;) print 1;");
    }

    #[test]
    fn for_nested() {
        compile("for (var i = 0; i < 2; i = i + 1) for (var j = 0; j < 2; j = j + 1) print i + j;");
    }

    #[test]
    fn dedups_repeated_number_literal() {
        let chunk = compile("print 1; print 1; print 1;");
        let numbers = chunk
            .constants
            .iter()
            .filter(|v| v.is_number())
            .count();
        assert_eq!(numbers, 1);
    }

    #[test]
    fn dedups_repeated_string_literal() {
        let chunk = compile(r#"print "hi"; print "hi"; print "hi";"#);
        let symbols = chunk
            .constants
            .iter()
            .filter(|v| v.is_symbol())
            .count();
        assert_eq!(symbols, 1);
    }

    #[test]
    fn dedups_repeated_global_reference() {
        // One number (1) + one identifier (a) — both reused across the three
        // statements. Without dedup we'd have 4 constants (1, a, a, a).
        let chunk = compile("var a = 1; print a; print a;");
        assert_eq!(chunk.constants.len(), 2);
    }

    #[test]
    fn distinct_numbers_get_distinct_slots() {
        let chunk = compile("print 1; print 2; print 3;");
        assert_eq!(chunk.constants.len(), 3);
    }
}
