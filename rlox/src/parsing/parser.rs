use std::{collections::VecDeque, fmt::Display};

use super::{error::ParsingError, expr::*, stmt::*};
use crate::{runtime::object::Object, tokens::*};

// program          => declaration* EOF ;
//
// declaration      => varDecl | statement ;
// statement        => exprStmt
//                  | printStmt
//                  | block
//                  | ifStmt;
// block            => "{" declaration* "}"
//
// varDecl          => "var" IDENTIFIER ( "=" expression )? ";" ;
// exprStmt         => expression ";" ;
// printStmt        => "print" expression ";" ;
// ifStmt           => "if" "(" expression ")" statement
//                  ( "else" statement )? ;
//
// expression       => assignment ;
// assignment       => IDENTIFIER "=" assignment | logic_or ;
// logic_or         => logic_and ( "or" logic_and )*
// logic_and        => equality ( "and" equality )*
// equality         => comparison ( ("!=" | "==") comparison )* ;
// comparison       => term ( (">" | ">=" | "<" | "<=") term )* ;
// term             => factor ( ("-" | "+") factor )* ;
// factor           => unary ( ("/" | "*") unary )* ;
// unary            => ("!" | "-") unary
//                  | primary ;
//
// primary          => NUMBER | STRING
//                  | "true" | "false" | "nil"
//                  | "(" expression ")"
//                  | IDENTIFIER ;

pub struct Parser {
    tokens: VecDeque<Token>,
    eof: Token,
}

/// Token type pattern matching helper
#[macro_export]
macro_rules! tt_pat {
    ($bind:ident @ $pat:pat) => {
        $bind @ $crate::lexing::tokens::Token {
            ty: $pat,
            ..
        }
    };
    [$pat:pat] => {
        $crate::lexing::tokens::Token {
            ty: $pat,
            ..
        }
    };
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        let eof = tokens.pop().expect("always have EOF token"); // None means EOF and we keep the token for reporting
        Self {
            tokens: tokens.into(),
            eof,
        }
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>, Vec<ParsingError>> {
        let mut stmts = vec![];
        let mut errs = vec![];
        while self.peek().is_some() {
            match self.declaration() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    self.synchronize();
                    errs.push(err);
                }
            };
        }
        if !errs.is_empty() {
            Err(errs)
        } else {
            Ok(stmts)
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek().map(Token::ty) {
            Some(TokenType::Var) => self.var_decl(),
            _ => self.statement(),
        }
    }

    fn var_decl(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenType::Var)?;
        let ident = self.consume_with(|t| matches!(t, TokenType::Identifier(_)), "identifier")?;
        let initializer = self
            .matches(TokenType::Equal)
            .map(|_| self.expression())
            .transpose()?;
        self.consume(TokenType::Semicolon)?;
        Ok(StmtVar { ident, initializer }.into())
    }

    fn statement(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek().map(Token::ty) {
            Some(TokenType::If) => self.if_stmt(),
            Some(TokenType::Print) => self.print_stmt(),
            Some(TokenType::LeftBrace) => Ok(StmtBlock {
                statements: self.block()?,
            }
            .into()),
            _ => self.expression_stmt(),
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParsingError> {
        self.consume(TokenType::LeftBrace)?;

        let mut statements = vec![];
        while self.peek().is_some_and(|t| t.ty != TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace)?;
        Ok(statements)
    }

    fn expression_stmt(&mut self) -> Result<Stmt, ParsingError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(StmtExpression { expr }.into())
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParsingError> {
        let print_token = self.consume(TokenType::Print)?;
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(StmtPrint { print_token, expr }.into())
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenType::If)?;
        self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;
        let then_branch = Box::new(self.statement()?);

        let else_branch = self
            .matches(TokenType::Else)
            .map(|_| self.statement().map(Box::new))
            .transpose()?;

        Ok(StmtIf {
            condition,
            then_branch,
            else_branch,
        }
        .into())
    }

    pub(crate) fn expression(&mut self) -> Result<Expr, ParsingError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParsingError> {
        let expr = self.logic_or()?;
        if let Some(ref equals) = self.matches(TokenType::Equal) {
            let value = Box::new(self.assignment()?);

            return match expr {
                Expr::Variable(ExprVariable { name }) => Ok(ExprAssign { name, value }.into()),
                _ => Err(ParsingError::custom(equals, "Invalid assignment target.")),
            };
        }
        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.logic_and()?;
        while let Some(op) = self.matches(TokenType::Or) {
            let right = self.logic_and()?;
            expr = ExprLogical {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.equality()?;
        while let Some(op) = self.matches(TokenType::And) {
            let right = self.equality()?;
            expr = ExprLogical {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.comparison()?;
        while let Some(op) =
            self.matches_with(|t| matches!(t, TokenType::BangEqual | TokenType::EqualEqual))
        {
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

    fn comparison(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.term()?;

        while let Some(op) = self.matches_with(|t| {
            matches!(
                t,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual,
            )
        }) {
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

    fn term(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.factor()?;
        while let Some(op) = self.matches_with(|t| matches!(t, TokenType::Minus | TokenType::Plus))
        {
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

    fn factor(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.unary()?;
        while let Some(op) = self.matches_with(|t| matches!(t, TokenType::Slash | TokenType::Star))
        {
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

    fn unary(&mut self) -> Result<Expr, ParsingError> {
        let expr = match self.matches_with(|t| matches!(t, TokenType::Bang | TokenType::Minus)) {
            Some(op) => ExprUnary {
                op,
                right: Box::new(self.unary()?),
            }
            .into(),
            None => self.primary()?,
        };
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParsingError> {
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
            Some(tt_pat!(ident @ TokenType::Identifier(_))) => ExprVariable { name: ident }.into(),
            Some(tok) => return Err(ParsingError::expected("expression", &tok)),
            None => return Err(ParsingError::expected("expression", &self.eof)),
        };
        Ok(expr)
    }

    fn synchronize(&mut self) {
        while let Some(tok) = self.advance() {
            match tok.ty() {
                TokenType::Semicolon => return,
                _ => match self.peek().map(Token::ty) {
                    Some(
                        TokenType::Class
                        | TokenType::For
                        | TokenType::Fun
                        | TokenType::If
                        | TokenType::Print
                        | TokenType::Return
                        | TokenType::Var
                        | TokenType::While,
                    ) => return,
                    _ => continue,
                },
            }
        }
    }

    fn matches(&mut self, patterns: TokenType) -> Option<Token> {
        match self.peek() {
            Some(tok) if patterns == tok.ty => {
                let tok = self
                    .advance()
                    .expect("peek has a value in this branch, it's safe to advance");
                Some(tok)
            }
            _ => None,
        }
    }

    fn matches_with(&mut self, pattern_fn: impl FnOnce(&TokenType) -> bool) -> Option<Token> {
        match self.peek() {
            Some(tok) if pattern_fn(&tok.ty) => {
                let tok = self
                    .advance()
                    .expect("peek has a value in this branch, it's safe to advance");
                Some(tok)
            }
            _ => None,
        }
    }

    fn consume(&mut self, pattern: TokenType) -> Result<Token, ParsingError> {
        match self.advance() {
            Some(tok) if pattern == tok.ty => Ok(tok),
            Some(tok) => Err(ParsingError::expected(pattern, &tok)),
            None => Err(ParsingError::expected(pattern, &self.eof)),
        }
    }

    fn consume_with(
        &mut self,
        pattern_fn: impl FnOnce(&TokenType) -> bool,
        expected_tok: impl Display,
    ) -> Result<Token, ParsingError> {
        match self.advance() {
            Some(tok) if pattern_fn(&tok.ty) => Ok(tok),
            Some(tok) => Err(ParsingError::expected(expected_tok, &tok)),
            None => Err(ParsingError::expected(expected_tok, &self.eof)),
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
    use crate::lexing::Scanner;

    #[test]
    fn parse_grouping() {
        let src = "(42)";
        let tokens = Scanner::new(src).scan_tokens().unwrap();
        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(expr.polish_notation(), "(group 42)")
    }

    #[test]
    fn parse_equality() {
        let src = "42 == 42 != 69 != 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(expr.polish_notation(), "(!= (!= (== 42 42) 69) 420)")
    }

    #[test]
    fn parse_comparison() {
        let src = "42 < 69 <= 69 > 13 >= 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(expr.polish_notation(), "(>= (> (<= (< 42 69) 69) 13) 420)");
    }

    #[test]
    fn parse_term() {
        let src = "42 - 69 + 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(expr.polish_notation(), "(+ (- 42 69) 420)");
    }

    #[test]
    fn parse_factor() {
        let src = "42 / 69 * 420";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(expr.polish_notation(), "(* (/ 42 69) 420)");
    }

    #[test]
    fn parse_unary() {
        let src = "!-42";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(expr.polish_notation(), "(! (- 42))");
    }

    #[test]
    fn test_precedence() {
        let src = "42 + -69 * 420 == (\"wtv\" > !false != nil)";
        let tokens = Scanner::new(src).scan_tokens().unwrap();

        let expr = Parser::new(tokens).expression().unwrap();

        assert_eq!(
            expr.polish_notation(),
            "(== (+ 42 (* (- 69) 420)) (group (!= (> \"wtv\" (! false)) nil)))"
        );
    }
}
