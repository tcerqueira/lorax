use std::fmt::{self, Debug, Display};

use super::{object::Object, visitor::Visitor};
use crate::{span::Span, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(ExprBinary),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprBinary {
    pub op: Token,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprGrouping(pub Box<Expr>);

#[derive(Debug, Clone)]
pub struct ExprLiteral {
    pub token: Token,
    pub literal: Object,
}

impl PartialEq for ExprLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn span(&self) -> Span {
        match self {
            Expr::Binary(e) => e.left.span().join(&e.right.span()),
            Expr::Grouping(e) => e.0.span(),
            Expr::Literal(e) => e.token.span.clone(),
            Expr::Unary(e) => e.op.span.join(&e.right.span()),
        }
    }

    #[cfg(test)]
    pub fn polish_notation(&self) -> String {
        struct PolishNotation<'a>(&'a Expr);
        impl Display for PolishNotation<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.accept(&mut AstPrinter { fmt: f })
            }
        }

        PolishNotation(self).to_string()
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.accept(&mut ReportPrinter { fmt: f })
    }
}

struct ReportPrinter<'a, 'f> {
    fmt: &'a mut fmt::Formatter<'f>,
}

impl Visitor for ReportPrinter<'_, '_> {
    type T = fmt::Result;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T {
        expr.left.accept(self)?;
        write!(self.fmt, " {} ", expr.op.ty)?;
        expr.right.accept(self)
    }

    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T {
        write!(self.fmt, "(")?;
        expr.0.accept(self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T {
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T {
        write!(self.fmt, "{}", expr.op.ty)?;
        expr.right.accept(self)
    }
}

pub struct AstPrinter<'a, 'f> {
    pub fmt: &'a mut fmt::Formatter<'f>,
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
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.right.accept(self)?;
        write!(self.fmt, ")")
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
    use super::*;
    use crate::tok;

    #[test]
    fn test_printer() {
        let expr: Expr = ExprBinary {
            op: tok![*],
            left: Box::new(
                ExprUnary {
                    op: tok![-],
                    right: Box::new(
                        ExprLiteral {
                            token: (tok![n:123]),
                            literal: Object::nil(),
                        }
                        .into(),
                    ),
                }
                .into(),
            ),
            right: Box::new(
                ExprGrouping(Box::new(
                    ExprLiteral {
                        token: (tok![n:45.67]),
                        literal: Object::nil(),
                    }
                    .into(),
                ))
                .into(),
            ),
        }
        .into();

        assert_eq!("(* (- 123) (group 45.67))", expr.polish_notation());
    }
}
