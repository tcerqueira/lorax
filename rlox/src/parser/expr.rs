use std::fmt::{self, Display, Formatter};

use super::visitor::Visitor;

use crate::tokens::Token;

pub enum Expr {
    Binary(ExprBinary),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
}

pub struct ExprBinary {
    pub op: Token,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

pub struct ExprGrouping {
    pub inner: Box<Expr>,
}

pub struct ExprLiteral {
    pub token: Token,
}

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

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.accept(&mut AstPrinter { fmt: f })
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::{Token, TokenType};

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
