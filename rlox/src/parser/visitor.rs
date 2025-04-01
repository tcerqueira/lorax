use std::fmt::{self, Formatter};

use super::*;

pub trait Visitor {
    type T;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T;
    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T;
    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T;
    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T;
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
        expr.inner.accept(self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T {
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.right.accept(self)?;
        write!(self.fmt, ")")
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::TokenType;

    use super::*;

    #[test]
    fn test_printer() {
        let expr = Expr::Binary(ExprBinary {
            op: Token::new(TokenType::Star),
            left: Box::new(Expr::Unary(ExprUnary {
                op: Token::new(TokenType::Minus),
                right: Box::new(Expr::Literal(ExprLiteral {
                    token: Token::new(TokenType::Number(123.)),
                })),
            })),
            right: Box::new(Expr::Grouping(ExprGrouping {
                inner: Box::new(Expr::Literal(ExprLiteral {
                    token: Token::new(TokenType::Number(45.67)),
                })),
            })),
        });

        assert_eq!("(* (- 123) (group 45.67))", format!("{}", expr));
    }
}
