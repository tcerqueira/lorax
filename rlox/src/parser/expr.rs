use std::fmt::{self, Display, Formatter};

use super::visitor::Visitor;
use crate::tokens::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(ExprBinary),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
}

#[derive(Debug, PartialEq)]
pub struct ExprBinary {
    pub op: Token,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct ExprGrouping(pub Box<Expr>);

#[derive(Debug, PartialEq)]
pub struct ExprLiteral(pub Token);

#[derive(Debug, PartialEq)]
pub struct ExprUnary {
    pub op: Token,
    pub right: Box<Expr>,
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

pub struct AstPrinter<'a, 'f> {
    pub fmt: &'a mut Formatter<'f>,
}

impl Visitor for AstPrinter<'_, '_> {
    type T = fmt::Result;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.left.accept(self)?;
        write!(self.fmt, " ")?;
        expr.right.accept(self)?;
        write!(self.fmt, ")")
    }

    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T {
        write!(self.fmt, "(group ")?;
        expr.0.accept(self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T {
        write!(self.fmt, "{}", expr.0.ty)
    }

    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.right.accept(self)?;
        write!(self.fmt, ")")
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.accept(&mut AstPrinter { fmt: f })
    }
}

impl From<ExprBinary> for Expr {
    fn from(value: ExprBinary) -> Self {
        Self::Binary(value)
    }
}

impl From<ExprUnary> for Expr {
    fn from(value: ExprUnary) -> Self {
        Self::Unary(value)
    }
}

impl From<ExprGrouping> for Expr {
    fn from(value: ExprGrouping) -> Self {
        Self::Grouping(value)
    }
}

impl From<ExprLiteral> for Expr {
    fn from(value: ExprLiteral) -> Self {
        Self::Literal(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tok,
        tokens::{Token, TokenType},
    };

    use super::*;

    #[test]
    fn test_printer() {
        let expr: Expr = ExprBinary {
            op: tok![*],
            left: Box::new(
                ExprUnary {
                    op: tok![-],
                    right: Box::new(ExprLiteral(tok![n:123]).into()),
                }
                .into(),
            ),
            right: Box::new(ExprGrouping(Box::new(ExprLiteral(tok![n:45.67]).into())).into()),
        }
        .into();

        assert_eq!("(* (- 123) (group 45.67))", format!("{}", expr));
    }
}
