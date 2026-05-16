use std::iter::Peekable;

use lexer::{
    Scanner,
    tokens::{Token, TokenType},
};
use report::error::ParsingError;
use report::{Span, error::LexingError};
use thiserror::Error;

use crate::{chunk::Chunk, opcode::OpCode, storage::Storage, value::Value, write_with_line};

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Lexing(#[from] LexingError),
    #[error(transparent)]
    Parsing(#[from] ParsingError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<CompileError> for report::Error {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::Lexing(e) => e.into(),
            CompileError::Parsing(e) => e.into(),
            CompileError::Other(e) => e.into(),
        }
    }
}

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
        // TokenType::Equal => (2, 1),
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
}

pub struct Compiler<'s, 'h> {
    scanner: Peekable<Scanner<'s>>,
    chunk: Chunk,
    storage: &'h mut Storage,
}

impl<'s, 'h> Compiler<'s, 'h> {
    pub fn new(scanner: Scanner<'s>, storage: &'h mut Storage) -> Self {
        Self {
            scanner: scanner.peekable(),
            chunk: Chunk::default(),
            storage,
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, CompileError> {
        self.expression()?;
        self.end();
        Ok(std::mem::take(&mut self.chunk))
    }

    pub fn end(&mut self) {
        self.chunk.write(OpCode::Return);
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
            _ => Err(ParsingError::expected(&tok, "expression", &tok).into()),
        }
    }

    #[expect(unused)]
    fn parse_postfix(&mut self, tok: Token, lhs: Handle) -> Result<Handle, CompileError> {
        unimplemented!("no postfix ops atm");
    }

    fn expression(&mut self) -> Result<(), CompileError> {
        let handle = self.parse_bp(0)?;
        self.materialize(handle);
        Ok(())
    }

    fn materialize(&mut self, handle: Handle) {
        match handle {
            Handle::Value => {}
        }
    }

    fn number(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let Token {
            ty: TokenType::Number(num),
            span,
        } = tok
        else {
            unreachable!("expected number token");
        };
        self.chunk
            .write_constant_with_line(span.line_start, Value::number(num));
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
        self.chunk
            .write_constant_with_line(span.line_start, Value::object(obj));
        Ok(Handle::Value)
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
            TokenType::Minus => self.chunk.write_with_line(line, OpCode::Neg),
            TokenType::Bang => self.chunk.write_with_line(line, OpCode::Not),
            _ => panic!("expected minus token as prefix"),
        };

        Ok(Handle::Value)
    }

    fn binary(&mut self, op: Token, lhs: Handle) -> Result<Handle, CompileError> {
        self.materialize(lhs);
        let (_l_bp, r_bp) = infix_bp(op.ty()).expect("expected infix op token");
        let rhs = self.parse_bp(r_bp)?;
        self.materialize(rhs);

        let line = op.span.line_start;
        #[rustfmt::skip]
        match op.ty() {
            TokenType::Plus => self.chunk.write_with_line(line, OpCode::Add),
            TokenType::Minus => self.chunk.write_with_line(line, OpCode::Sub),
            TokenType::Star => self.chunk.write_with_line(line, OpCode::Mul),
            TokenType::Slash => self.chunk.write_with_line(line, OpCode::Div),
            TokenType::BangEqual => write_with_line!(self.chunk, line, OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.chunk.write_with_line(line, OpCode::Equal),
            TokenType::Greater => self.chunk.write_with_line(line, OpCode::Greater),
            TokenType::GreaterEqual => write_with_line!(self.chunk, line, OpCode::Less, OpCode::Not),
            TokenType::Less => self.chunk.write_with_line(line, OpCode::Less),
            TokenType::LessEqual => write_with_line!(self.chunk, line, OpCode::Greater, OpCode::Not),
            _ => panic!("expected op tokens: + - * /"),
        };
        Ok(Handle::Value)
    }

    fn literal(&mut self, tok: Token) -> Result<Handle, CompileError> {
        let line = tok.span.line_start;
        match tok.ty() {
            TokenType::True => self.chunk.write_with_line(line, OpCode::True),
            TokenType::False => self.chunk.write_with_line(line, OpCode::False),
            TokenType::Nil => self.chunk.write_with_line(line, OpCode::Nil),
            _ => panic!("expected literal tokens"),
        }
        Ok(Handle::Value)
    }

    fn advance(&mut self) -> Result<Option<Token>, CompileError> {
        self.scanner.next().transpose().map_err(Into::into)
    }

    fn peek(&mut self) -> Result<Option<&Token>, CompileError> {
        self.scanner
            .peek()
            .map(|res| res.as_ref().map_err(|err| err.clone().into()))
            .transpose()
    }

    fn consume(&mut self, pattern: TokenType) -> Result<Option<Token>, CompileError> {
        match self.advance()? {
            Some(tok) if pattern == tok.ty => Ok(Some(tok)),
            Some(tok) => Err(ParsingError::expected(&tok, pattern, &tok).into()),
            None => Err(ParsingError::expected(Span::default(), pattern, TokenType::Eof).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(src: &str) -> Chunk {
        let mut storage = Storage::new();
        Compiler::new(Scanner::new(src), &mut storage)
            .compile()
            .unwrap()
    }

    #[test]
    fn challenge() {
        let _program = compile("(-1 + 2) * 3 - -4");
    }

    #[test]
    fn arithmetic() {
        let _program = compile("2 * 3 + 4");
    }

    #[test]
    fn prefix() {
        let _program = compile("-2 * 3 + 4");
    }

    #[test]
    fn grouping() {
        let _program = compile("2 * (3 + 4)");
    }

    #[test]
    fn logical() {
        let _program = compile("!(5 - 4 > 3 * 2 == !nil)");
    }

    #[test]
    fn string_literal() {
        let _program = compile("\"hello\"");
    }

    #[test]
    fn string_ops() {
        let _program = compile("\"a\" + \"b\" == \"ab\"");
    }
}
