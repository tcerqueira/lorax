pub mod expr;
pub mod object;
pub mod visitor;

use std::collections::VecDeque;

use crate::{error::CompileError, tokens::*};
use expr::*;
use object::Object;

// expression   => equality;
// equality     => comparison ( ("!=" | "==") comparison )*;
// comparison   => term ( (">" | ">=" | "<" | "<=") term )*;
// term         => factor ( ("-" | "+") factor )*;
// factor       => unary ( ("/" | "*") unary )*;
// unary        => ("!" | "-") unary
//              | primary;
// primary      => NUMBER
//              | STRING
//              | "true"
//              | "false"
//              | "nil"
//              | "(" expression ")";

pub struct Parser<'s> {
    src: &'s str,
    tokens: VecDeque<Token>,
    eof: Token,
}

/// Token type pattern matching helper
#[macro_export]
macro_rules! tt_pat {
    ($bind:ident @ $pat:pat) => {
        $bind @ Token {
            ty: $pat,
            ..
        }
    };
    [$pat:pat] => {
        Token {
            ty: $pat,
            ..
        }
    };
}

impl<'s> Parser<'s> {
    pub fn parse(source: &'s str, mut tokens: Vec<Token>) -> Result<Expr, CompileError> {
        let eof = tokens.pop().expect("always have EOF token"); // None means EOF and we keep the token for reporting
        let mut parser = Self {
            src: source,
            tokens: tokens.into(),
            eof,
        };
        parser.expression().inspect_err(|e| eprintln!("{e}"))
    }

    fn expression(&mut self) -> Result<Expr, CompileError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.comparison()?;
        while let Some(op) = self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let right = self.comparison()?;
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.term()?;

        while let Some(op) = self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term()?;
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.factor()?;
        while let Some(op) = self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let right = self.factor()?;
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.unary()?;
        while let Some(op) = self.matches(&[TokenType::Slash, TokenType::Star]) {
            let right = self.unary()?;
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, CompileError> {
        let expr = match self.matches(&[TokenType::Bang, TokenType::Minus]) {
            Some(op) => ExprUnary {
                op,
                right: Box::new(self.unary()?),
            }
            .into(),
            None => self.primary()?,
        };
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, CompileError> {
        let expr = match self.advance() {
            Some(
                tt_pat!(token @ TokenType::Number(_) | TokenType::String(_) | TokenType::True | TokenType::False | TokenType::Nil),
            ) => {
                let literal = match token.ty {
                    TokenType::Number(n) => Object::new(n),
                    TokenType::String(ref s) => Object::new(String::from(s.as_ref())),
                    TokenType::True => Object::new(true),
                    TokenType::False => Object::new(false),
                    TokenType::Nil => Object::nil(),
                    _ => unreachable!("matched these variants before"),
                };
                ExprLiteral { token, literal }.into()
            }
            Some(tt_pat!(TokenType::LeftParen)) => {
                let inner = Box::new(self.expression()?);
                let expr = ExprGrouping(inner).into();
                self.consume(TokenType::RightParen)?;
                expr
            }
            Some(tok) => return Err(CompileError::expected(self.src, "expression", &tok)),
            None => return Err(CompileError::expected(self.src, "expression", &self.eof)),
        };
        Ok(expr)
    }

    #[expect(dead_code)]
    fn synchronize(&mut self) {
        while let Some(tok) = self.advance() {
            match tok {
                tt_pat!(TokenType::Semicolon) => return,
                _ => match self.peek() {
                    Some(
                        tt_pat!(
                            TokenType::Class
                                | TokenType::For
                                | TokenType::Fun
                                | TokenType::If
                                | TokenType::Print
                                | TokenType::Return
                                | TokenType::Var
                                | TokenType::While
                        ),
                    ) => return,
                    _ => continue,
                },
            }
        }
    }

    fn matches(&mut self, patterns: &[TokenType]) -> Option<Token> {
        match self.peek() {
            Some(tok) if patterns.contains(&tok.ty) => {
                let tok = self
                    .advance()
                    .expect("peek has a value in this branch, it's safe to advance");
                Some(tok)
            }
            _ => None,
        }
    }

    fn consume(&mut self, pattern: TokenType) -> Result<Token, CompileError> {
        match self.advance() {
            Some(tok) if pattern == tok.ty => Ok(tok),
            Some(tok) => Err(CompileError::expected(self.src, pattern, &tok)),
            None => Err(CompileError::expected(self.src, pattern, &self.eof)),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    #[test]
    fn parse_grouping() {
        let src = "(42)";
        let tokens = Scanner::new(src).scan_tokens().unwrap();
        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(expr.polish_notation(), "(group 42)")
    }

    #[test]
    fn parse_equality() {
        let src = "42 == 42 != 69 != 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(expr.polish_notation(), "(!= (!= (== 42 42) 69) 420)")
    }

    #[test]
    fn parse_comparison() {
        let src = "42 < 69 <= 69 > 13 >= 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(expr.polish_notation(), "(>= (> (<= (< 42 69) 69) 13) 420)");
    }

    #[test]
    fn parse_term() {
        let src = "42 - 69 + 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(expr.polish_notation(), "(+ (- 42 69) 420)");
    }

    #[test]
    fn parse_factor() {
        let src = "42 / 69 * 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(expr.polish_notation(), "(* (/ 42 69) 420)");
    }

    #[test]
    fn parse_unary() {
        let src = "!-42";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(expr.polish_notation(), "(! (- 42))");
    }

    #[test]
    fn test_precedence() {
        let src = "42 + -69 * 420 == (\"wtv\" > !false != nil)";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::parse(src, tokens).unwrap();

        assert_eq!(
            expr.polish_notation(),
            "(== (+ 42 (* (- 69) 420)) (group (!= (> \"wtv\" (! false)) nil)))"
        );
    }
}
