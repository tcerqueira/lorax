use std::{fmt::Display, iter::Peekable};

use anyhow::bail;
use lexer::{
    Scanner,
    tokens::{Token, TokenType},
};
use report::{Reporter, error::ParsingError};
use report::{Span, error::LexingError};

use crate::{
    chunk::Chunk,
    compiler::error::CompileError,
    opcode::OpCode,
    storage::Storage,
    value::{Addr, Value},
};

pub mod error;

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
        // TokenType::Or => (3, 4),
        // TokenType::And => (5, 6),
        TokenType::EqualEqual | TokenType::BangEqual => (7, 8),
        TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual => {
            (9, 10)
        }
        TokenType::Plus | TokenType::Minus => (11, 12),
        TokenType::Star | TokenType::Slash => (13, 14),
        // TokenType::Dot | TokenType::LeftParen => (16, 15),
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy)]
enum Handle {
    Value,
    Place(Place),
}

impl Handle {
    fn global(addr: Addr, line: u32) -> Self {
        Self::Place(Place::Global { addr, line })
    }
}

#[derive(Debug, Clone, Copy)]
enum Place {
    Global { addr: Addr, line: u32 },
}

pub struct Compiler<'s, 't> {
    scanner: Peekable<Scanner<'s>>,
    reporter: Reporter<'s>,
    chunk: Chunk,
    storage: &'t mut Storage,
    errored: bool,
}

impl<'s, 't> Compiler<'s, 't> {
    pub fn new(scanner: Scanner<'s>, reporter: Reporter<'s>, storage: &'t mut Storage) -> Self {
        Self {
            scanner: scanner.peekable(),
            reporter,
            chunk: Chunk::default(),
            storage,
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
            false => Ok(std::mem::take(&mut self.chunk)),
        }
    }

    pub fn end(&mut self) {
        self.emit_op(OpCode::Return);
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
        let (tok, global) = self.parse_variable()?;

        match self.advance_if(TokenType::Equal)? {
            Some(_) => self.expression()?,
            None => self.emit_op_and_line(tok.span.line_start, OpCode::Nil),
        }
        let tok = self.consume(TokenType::Semicolon)?;

        self.define_variable(tok, global);
        Ok(())
    }

    fn statement(&mut self) -> Result<(), CompileError> {
        let Some(tok) = self.peek()? else {
            return Ok(());
        };
        match tok.ty() {
            TokenType::Print => self.print_stmt(),
            _ => self.expression_stmt(),
        }
    }

    fn print_stmt(&mut self) -> Result<(), CompileError> {
        let tok = self
            .consume(TokenType::Print)
            .expect("matched print token before entering this branch");
        self.expression()?;
        self.consume(TokenType::Semicolon)?;
        self.emit_op_and_line(tok.span.line_start, OpCode::Print);
        Ok(())
    }

    fn expression_stmt(&mut self) -> Result<(), CompileError> {
        self.expression()?;
        self.consume(TokenType::Semicolon)?;
        self.emit_op(OpCode::Pop);
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

        let line = op.span.line_start;
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
        let obj = self.storage.add_internal_str(&s);
        self.emit_constant_and_line(span.line_start, Value::object(obj));
        Ok(Handle::Value)
    }

    fn literal(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let line = tok.span.line_start;
        match tok.ty() {
            TokenType::True => self.emit_op_and_line(line, OpCode::True),
            TokenType::False => self.emit_op_and_line(line, OpCode::False),
            TokenType::Nil => self.emit_op_and_line(line, OpCode::Nil),
            _ => panic!("expected literal tokens"),
        }
        Ok(Handle::Value)
    }

    fn named_variable(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let addr = self.ident_constant(&tok);
        Ok(Handle::global(addr, tok.span.line_start))
    }

    fn binary(&mut self, op: Token, lhs: Handle) -> Result<Handle, CompileError> {
        self.materialize(lhs);
        let (_l_bp, r_bp) = infix_bp(op.ty()).expect("expected infix op token");
        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);

        let line = op.span.line_start;
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
        }
    }

    fn store(&mut self, place: Place) {
        match place {
            Place::Global { addr, line } => {
                self.emit_op_and_line(line, OpCode::SetGlobal(addr));
            }
        }
    }

    fn ident_constant(&mut self, tok: &Token) -> Addr {
        let obj_ref = self.storage.add_internal_str(tok.as_str().as_ref());
        self.chunk.add_constant(Value::object(obj_ref))
    }

    fn parse_variable(&mut self) -> Result<(Token, Addr), CompileError> {
        let ident = self.consume_with(
            |t| matches!(t, TokenType::Identifier(_)),
            "variable identifier",
        )?;
        let addr = self.ident_constant(&ident);
        Ok((ident, addr))
    }

    fn define_variable(&mut self, semicolon: Token, addr: Addr) {
        self.emit_op_and_line(semicolon.span.line_start, OpCode::DefineGlobal(addr));
    }

    fn emit_op(&mut self, op: OpCode) {
        self.chunk.write(op);
    }

    fn emit_op_and_line(&mut self, line: u32, op: OpCode) {
        self.chunk.write_with_line(line, op);
    }

    fn emit_constant_and_line(&mut self, line: u32, value: Value) -> Addr {
        self.chunk.write_constant_with_line(line, value)
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
            .unwrap()
    }

    #[test]
    fn challenge() {
        let _program = compile("(-1 + 2) * 3 - -4;");
    }

    #[test]
    fn arithmetic() {
        let _program = compile("2 * 3 + 4;");
    }

    #[test]
    fn prefix() {
        let _program = compile("-2 * 3 + 4;");
    }

    #[test]
    fn grouping() {
        let _program = compile("2 * (3 + 4);");
    }

    #[test]
    fn logical() {
        let _program = compile("!(5 - 4 > 3 * 2 == !nil);");
    }

    #[test]
    fn string_literal() {
        let _program = compile("\"hello\";");
    }

    #[test]
    fn string_ops() {
        let _program = compile("\"a\" + \"b\" == \"ab\";");
    }

    #[test]
    fn var_declaration() {
        let _program = compile("var a = 1 + 2; print a;");
    }

    #[test]
    fn var_assignment() {
        let _program = compile("var a = 1 + 2; a = 0;");
    }
}
