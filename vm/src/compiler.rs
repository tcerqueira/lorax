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
    compiler::{context::LexicalContext, error::CompileError},
    enconding::{Addr, LocalSlot, OpCode},
    storage::Storage,
    value::Value,
};

pub mod context;
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
        // TokenType::Dot => (15, 16),
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

    fn local(slot: LocalSlot, line: u32) -> Self {
        Self::Place(Place::Local { slot, line })
    }
}

#[derive(Debug, Clone, Copy)]
enum Place {
    Global { addr: Addr, line: u32 },
    Local { slot: LocalSlot, line: u32 },
}

pub struct Compiler<'s, 't> {
    scanner: Peekable<Scanner<'s>>,
    reporter: Reporter<'s>,
    storage: &'t mut Storage,
    context: LexicalContext,
    errored: bool,
}

impl<'s, 't> Compiler<'s, 't> {
    pub fn new(scanner: Scanner<'s>, reporter: Reporter<'s>, storage: &'t mut Storage) -> Self {
        Self {
            scanner: scanner.peekable(),
            reporter,
            storage,
            context: LexicalContext::default(),
            errored: false,
        }
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
            false => Ok(std::mem::take(self.context.chunk_mut())),
        }
    }

    pub fn end(&mut self) {
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
            _ => self.statement(),
        }
    }

    fn var_decl(&mut self) -> Result<(), CompileError> {
        self.consume(TokenType::Var)?;
        let ident = self.consume_with(
            |t| matches!(t, TokenType::Identifier(_)),
            "variable identifier",
        )?;

        if self.context.at_global() {
            self.global_var_decl(ident)
        } else {
            self.local_var_decl(ident)
        }
    }

    fn global_var_decl(&mut self, ident: Token) -> Result<(), CompileError> {
        let name = self.storage.intern(&ident.as_str());
        let addr = self.ident_constant(name);
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
        self.context
            .scopes_mut()
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

    fn statement(&mut self) -> Result<(), CompileError> {
        let Some(tok) = self.peek()? else {
            return Ok(());
        };
        match tok.ty() {
            TokenType::Print => self.print_stmt(),
            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::For => self.for_stmt(),
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

        let then_jmp = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalse(0));
        self.emit_pops(1);
        self.statement()?;

        let else_jmp = self.emit_jmp(OpCode::Jmp(0));
        self.patch_jmp(then_jmp);
        self.emit_pops(1);

        if self.advance_if(TokenType::Else)?.is_some() {
            self.statement()?;
        }
        self.patch_jmp(else_jmp);

        Ok(())
    }

    fn while_stmt(&mut self) -> Result<(), CompileError> {
        let loop_start = self.context.chunk().current();
        self.consume(TokenType::While)
            .expect("matched token before entering this branch");
        self.consume(TokenType::LeftParen)
            .context("Expect '(' after 'while'.")?;
        self.expression()?;
        let tok = self
            .consume(TokenType::RightParen)
            .context("Expect ')' after 'condition'.")?;

        let exit_jmp = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalse(0));
        self.emit_op_and_line(tok.line(), OpCode::Pop);
        self.statement()?;
        self.emit_loop(loop_start);

        self.patch_jmp(exit_jmp);
        self.emit_pops(1);
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

        let mut loop_start = this.context.chunk().current();
        let exit_jmp = if this.advance_if(TokenType::Semicolon)?.is_none() {
            this.expression()?;
            let tok = this
                .consume(TokenType::Semicolon)
                .context("Expect ';' after a loop condition.")?;
            let exit_jmp = this.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalse(0));
            this.emit_pops(1);
            Some(exit_jmp)
        } else {
            None
        };

        if this.advance_if(TokenType::RightParen)?.is_none() {
            let body_jmp = this.emit_jmp(OpCode::Jmp(0));
            let inc_start = this.context.chunk().current();
            this.expression()?;
            this.emit_pops(1);
            this.consume(TokenType::RightParen)
                .context("Expect ')' after for clauses.")?;

            this.emit_loop(loop_start);
            loop_start = inc_start;
            this.patch_jmp(body_jmp);
        }

        this.statement()?;
        this.emit_loop(loop_start);

        if let Some(exit_jmp) = exit_jmp {
            this.patch_jmp(exit_jmp);
            this.emit_pops(1);
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
            _ => Err(ParsingError::expected(&tok, "expression", &tok).into()),
        }
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
        self.emit_constant_and_line(span.line_start, Value::number(num));
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
        self.emit_constant_and_line(span.line_start, Value::symbol(key));
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
        let line = tok.line();
        // Locals shadow globals.
        if let Some(slot) = self.context.scopes().resolve(name) {
            return Ok(Handle::local(slot, line));
        }
        let addr = self.ident_constant(name);
        Ok(Handle::global(addr, line))
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
        self.patch_jmp(short_circuit);

        Ok(Handle::Value)
    }

    fn or(&mut self, tok: Token, lhs: Handle) -> Result<Handle, CompileError> {
        let (_l_bp, r_bp) = infix_bp(tok.ty()).expect("expected infix op token");
        self.materialize(lhs);

        let else_jmp = self.emit_jmp_and_line(tok.line(), OpCode::JmpIfFalse(0));
        let end_jmp = self.emit_jmp_and_line(tok.line(), OpCode::Jmp(0));

        self.patch_jmp(else_jmp);
        self.emit_op_and_line(tok.line(), OpCode::Pop);

        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);
        self.patch_jmp(end_jmp);

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

    fn materialize(&mut self, handle: Handle) {
        match handle {
            Handle::Value => {}
            Handle::Place(Place::Global { addr, line }) => {
                self.emit_op_and_line(line, OpCode::GetGlobal(addr));
            }
            Handle::Place(Place::Local { slot, line }) => {
                self.emit_op_and_line(line, OpCode::GetLocal(slot));
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
        }
    }

    fn ident_constant(&mut self, name: Spur) -> Addr {
        self.add_constant(Value::symbol(name))
    }

    #[must_use]
    fn begin_scope<'c>(
        &'c mut self,
    ) -> ScopeGuard<&'c mut Compiler<'s, 't>, impl FnOnce(&'c mut Compiler<'s, 't>)> {
        self.context.scopes_mut().enter();
        scopeguard::guard(self, |this| {
            let pop_count = this.context.scopes_mut().exit();
            this.emit_pops(pop_count);
        })
    }

    fn emit_pops(&mut self, count: usize) {
        debug_assert!(count <= u8::MAX as usize, "Scopes caps locals at u8::MAX");
        match count {
            0 => {}
            1 => self.emit_op(OpCode::Pop),
            n => self.emit_op(OpCode::PopN(n as u8)),
        }
    }

    fn emit_op(&mut self, op: OpCode) {
        self.context.chunk_mut().write(op);
    }

    fn emit_op_and_line(&mut self, line: u32, op: OpCode) {
        self.context.chunk_mut().write_with_line(line, op);
    }

    fn add_constant(&mut self, value: Value) -> Addr {
        if let Some(addr) = self
            .context
            .chunk()
            .constants
            .iter()
            .rposition(|v| v == &value)
        {
            return addr as Addr;
        }
        self.context.chunk_mut().add_constant(value)
    }

    fn emit_constant_and_line(&mut self, line: u32, value: Value) -> Addr {
        let addr = self.add_constant(value);
        self.context
            .chunk_mut()
            .write_with_line(line, OpCode::Constant(addr));
        addr
    }

    fn emit_jmp_and_line(&mut self, line: u32, op: OpCode) -> u64 {
        self.emit_op_and_line(line, op);
        self.context.chunk().current()
    }

    fn emit_jmp(&mut self, op: OpCode) -> u64 {
        self.emit_op(op);
        self.context.chunk().current()
    }

    fn emit_loop(&mut self, loop_start: u64) {
        let offset = self.context.chunk().current() - loop_start + 3; // sizeof(OP_LOOP) = 3
        assert!(offset <= u16::MAX as u64, "Loop body too large.");
        self.emit_op(OpCode::Loop(offset as u16));
    }

    fn patch_jmp(&mut self, offset: u64) {
        let jmp = self.context.chunk().current() - offset;
        assert!(jmp <= u16::MAX as u64, "too much code to jump over.");
        self.context
            .chunk_mut()
            .write_raw(offset - 2, &(jmp as u16).to_le_bytes());
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
            .filter(|v| matches!(v, Value::Number(_)))
            .count();
        assert_eq!(numbers, 1);
    }

    #[test]
    fn dedups_repeated_string_literal() {
        let chunk = compile(r#"print "hi"; print "hi"; print "hi";"#);
        let symbols = chunk
            .constants
            .iter()
            .filter(|v| matches!(v, Value::Symbol(_)))
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
