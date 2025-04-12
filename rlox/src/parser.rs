mod expr;
mod visitor;

use std::{any::Any, collections::VecDeque};

use crate::{error::CompileError, tokens::*};
use expr::*;

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

pub struct Parser {
    tokens: VecDeque<Token>,
}

macro_rules! token_pat {
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

impl Parser {
    pub fn parse(mut tokens: Vec<Token>) -> Expr {
        tokens.pop(); // ignore EOF
        let mut parser = Self {
            tokens: tokens.into(),
        };
        parser.expression()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while let Some(op) = self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let right = self.comparison();
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while let Some(op) = self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term();
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while let Some(op) = self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let right = self.factor();
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while let Some(op) = self.matches(&[TokenType::Slash, TokenType::Star]) {
            let right = self.unary();
            expr = ExprBinary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        match self.matches(&[TokenType::Bang, TokenType::Minus]) {
            Some(op) => ExprUnary {
                op,
                right: Box::new(self.unary()),
            }
            .into(),
            None => self.primary(),
        }
    }

    fn primary(&mut self) -> Expr {
        match self.advance() {
            Some(
                token_pat!(token @ TokenType::Number(_) | TokenType::String(_) | TokenType::True | TokenType::False | TokenType::Nil),
            ) => {
                let literal: Option<Box<dyn Any>> = match token.ty {
                    TokenType::Number(n) => Some(Box::new(n)),
                    TokenType::String(ref s) => Some(Box::new(String::from(s.as_ref()))),
                    TokenType::True => Some(Box::new(true)),
                    TokenType::False => Some(Box::new(false)),
                    TokenType::Nil => None,
                    _ => unreachable!("matched these variants before"),
                };
                ExprLiteral { token, literal }.into()
            }
            Some(token_pat!(TokenType::LeftParen)) => {
                let inner = Box::new(self.expression());
                let expr = ExprGrouping(inner).into();
                match self.advance() {
                    Some(token_pat!(TokenType::RightParen)) => {}
                    Some(tok) => {
                        eprintln!("{}", CompileError::expected(TokenType::RightParen, tok))
                    }
                    None => eprintln!(
                        "{}",
                        CompileError {
                            line: 0,
                            span: "".into(),
                            message: "Expected ')', found end of file".into(),
                        }
                    ),
                }
                expr
            }
            _ => ExprLiteral {
                token: Token {
                    ty: TokenType::Eof,
                    span: "".into(),
                    line: 0,
                },
                literal: None,
            }
            .into(),
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
    use crate::tok;

    #[test]
    fn parse_grouping() {
        let tokens = vec![tok!['('], tok![n:42], tok![')'], tok![EOF]];
        let expr = Parser::parse(tokens);

        assert_eq!(expr.to_string(), "(group 42)")
    }

    #[test]
    fn parse_equality() {
        let tokens = vec![
            tok![n:42],
            tok![==],
            tok![n:42],
            tok![!=],
            tok![n:69],
            tok![!=],
            tok![n:420],
            tok![EOF],
        ];
        let expr = Parser::parse(tokens);

        assert_eq!(expr.to_string(), "(!= (!= (== 42 42) 69) 420)")
    }

    #[test]
    fn parse_comparison() {
        let tokens = vec![
            tok![n:42],
            tok![<],
            tok![n:69],
            tok![<=],
            tok![n:69],
            tok![>],
            tok![n:13],
            tok![>=],
            tok![n:420],
            tok![EOF],
        ];
        let expr = Parser::parse(tokens);

        assert_eq!(expr.to_string(), "(>= (> (<= (< 42 69) 69) 13) 420)");
    }

    #[test]
    fn parse_term() {
        let tokens = vec![
            tok![n:42],
            tok![-],
            tok![n:69],
            tok![+],
            tok![n:420],
            tok![EOF],
        ];
        let expr = Parser::parse(tokens);

        assert_eq!(expr.to_string(), "(+ (- 42 69) 420)");
    }

    #[test]
    fn parse_factor() {
        let tokens = vec![
            tok![n:42],
            tok![/],
            tok![n:69],
            tok![*],
            tok![n:420],
            tok![EOF],
        ];
        let expr = Parser::parse(tokens);

        assert_eq!(expr.to_string(), "(* (/ 42 69) 420)");
    }

    #[test]
    fn parse_unary() {
        let tokens = vec![tok![!], tok![-], tok![n:42], tok![EOF]];
        let expr = Parser::parse(tokens);

        assert_eq!(expr.to_string(), "(! (- 42))");
    }

    #[test]
    fn test_precedence() {
        let tokens = vec![
            tok![n:42],
            tok![+],
            tok![-],
            tok![n:69],
            tok![*],
            tok![n:420],
            tok![==],
            tok!['('],
            tok![s:"wtv"],
            tok![>],
            tok![!],
            tok![false],
            tok![!=],
            tok![nil],
            tok![')'],
            tok![EOF],
        ];
        let expr = Parser::parse(tokens);

        assert_eq!(
            expr.to_string(),
            "(== (+ 42 (* (- 69) 420)) (group (!= (> \"wtv\" (! false)) nil)))"
        );
    }
}
