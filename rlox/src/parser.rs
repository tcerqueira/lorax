use std::fmt::Display;

use visitor::{AstPrinter, Visitor};

use crate::tokens::Token;

mod visitor;

pub enum Expr {
    Binary(ExprBinary),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
}

pub struct ExprBinary {
    op: Token,
    left: Box<Expr>,
    right: Box<Expr>,
}

pub struct ExprGrouping {
    inner: Box<Expr>,
}

pub struct ExprLiteral {
    token: Token,
}

pub struct ExprUnary {
    op: Token,
    right: Box<Expr>,
}

impl Expr {
    pub fn accept<R>(&self, visitor: &mut impl Visitor<T = R>) -> R {
        match self {
            Expr::Binary(expr_binary) => visitor.visit_binary(expr_binary),
            Expr::Grouping(expr_grouping) => visitor.visit_grouping(expr_grouping),
            Expr::Literal(expr_literal) => visitor.visit_literal(expr_literal),
            Expr::Unary(expr_unary) => visitor.visit_unary(expr_unary),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.accept(&mut AstPrinter { fmt: f })
    }
}
