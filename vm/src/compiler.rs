use std::iter::Peekable;

use lexer::{
    Scanner,
    tokens::{Token, TokenType},
};
use report::error::ParsingError;
use report::{Span, error::LexingError};
use thiserror::Error;

use crate::{chunk::Chunk, opcode::OpCode, value::Value};

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
        // TokenType::EqualEqual | TokenType::BangEqual => (7, 8),
        // TokenType::Less
        // | TokenType::LessEqual
        // | TokenType::Greater
        // | TokenType::GreaterEqual => (9, 10),
        TokenType::Plus | TokenType::Minus => (11, 12),
        TokenType::Star | TokenType::Slash => (13, 14),
        // TokenType::Dot | TokenType::LeftParen => (16, 15),
        _ => return None,
    })
}

pub struct Compiler<'s> {
    scanner: Peekable<Scanner<'s>>,
    chunk: Chunk,
}

impl<'s> Compiler<'s> {
    pub fn new(scanner: Scanner<'s>) -> Self {
        Self {
            scanner: scanner.peekable(),
            chunk: Chunk::default(),
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

    fn parse_bp(&mut self, min_bp: u8) -> Result<(), CompileError> {
        let lhs = self
            .advance()?
            .ok_or(ParsingError::expected(Span::default(), "token", "EOF"))?;
        self.parse_prefix(lhs)?;

        loop {
            let op = match self.peek()? {
                None => break,
                Some(tok) => tok,
            };

            match op_bp(&op.ty) {
                Some(OpBinding::Postfix(l_bp)) => {
                    if l_bp < min_bp {
                        break;
                    }
                    let op = self.advance().unwrap().unwrap();
                    self.parse_postfix(op)?;
                    continue;
                }
                Some(OpBinding::Infix(l_bp, _r_bp)) => {
                    if l_bp < min_bp {
                        break;
                    }
                    let op = self.advance().unwrap().unwrap();
                    self.parse_infix(op)?;
                    continue;
                }
                None => break,
            };
        }

        Ok(())
    }

    fn parse_prefix(&mut self, tok: Token) -> Result<(), CompileError> {
        match tok.ty() {
            TokenType::LeftParen => self.grouping(tok),
            TokenType::Minus => self.unary(tok),
            TokenType::Number(_) => self.number(tok),
            TokenType::True => self.literal(tok),
            TokenType::False => self.literal(tok),
            TokenType::Nil => self.literal(tok),
            _ => Err(ParsingError::expected(&tok, "expression", &tok).into()),
        }
    }

    fn parse_infix(&mut self, tok: Token) -> Result<(), CompileError> {
        match tok.ty() {
            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => {
                self.binary(tok)
            }
            _ => Err(ParsingError::expected(&tok, "expression", &tok).into()),
        }
    }

    #[expect(unused)]
    fn parse_postfix(&mut self, tok: Token) -> Result<(), CompileError> {
        unimplemented!("no postfix ops atm");
    }

    fn expression(&mut self) -> Result<(), CompileError> {
        self.parse_bp(0)
    }

    fn number(&mut self, tok: Token) -> Result<(), CompileError> {
        let Token {
            ty: TokenType::Number(num),
            span,
        } = tok
        else {
            panic!("expected number token");
        };
        self.chunk
            .write_constant_with_line(Value::number(num), span.line_start);
        Ok(())
    }

    fn grouping(&mut self, _tok: Token) -> Result<(), CompileError> {
        self.expression()?;
        self.consume(TokenType::RightParen)?;
        Ok(())
    }

    fn unary(&mut self, op: Token) -> Result<(), CompileError> {
        let r_bp = prefix_bp(op.ty()).expect("expected infix op token");
        self.parse_bp(r_bp)?;
        assert_eq!(op.ty(), &TokenType::Minus, "expected minus token as prefix");
        self.chunk.write_with_line(OpCode::Neg, op.span.line_start);
        Ok(())
    }

    fn binary(&mut self, op: Token) -> Result<(), CompileError> {
        let (_l_bp, r_bp) = infix_bp(op.ty()).expect("expected infix op token");
        self.parse_bp(r_bp)?;

        let line = op.span.line_start;
        match op.ty() {
            TokenType::Plus => self.chunk.write_with_line(OpCode::Add, line),
            TokenType::Minus => self.chunk.write_with_line(OpCode::Sub, line),
            TokenType::Star => self.chunk.write_with_line(OpCode::Mul, line),
            TokenType::Slash => self.chunk.write_with_line(OpCode::Div, line),
            _ => panic!("expected op tokens: + - * /"),
        }
        Ok(())
    }

    fn literal(&mut self, tok: Token) -> Result<(), CompileError> {
        let line = tok.span.line_start;
        match tok.ty() {
            TokenType::True => self.chunk.write_with_line(OpCode::True, line),
            TokenType::False => self.chunk.write_with_line(OpCode::False, line),
            TokenType::Nil => self.chunk.write_with_line(OpCode::Nil, line),
            _ => panic!("expected literal tokens"),
        }
        Ok(())
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
    use lexer::Scanner;

    use crate::{chunk::Chunk, compiler::Compiler};

    fn compile(src: &str) -> Chunk {
        Compiler::new(Scanner::new(src)).compile().unwrap()
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
}
