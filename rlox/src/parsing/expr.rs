use std::fmt::{self, Debug, Display};

use super::visitor::ExprVisitor;
use crate::{report::Span, runtime::object::Object, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(ExprBinary),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
    Variable(ExprVariable),
    Assign(ExprAssign),
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

#[derive(Debug, Clone, PartialEq)]
pub struct ExprVariable {
    pub name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign {
    pub name: Token,
    pub value: Box<Expr>,
}

impl Expr {
    pub fn accept<R>(&self, visitor: &mut impl ExprVisitor<T = R>) -> R {
        match self {
            Expr::Binary(e) => visitor.visit_binary(e),
            Expr::Grouping(e) => visitor.visit_grouping(e),
            Expr::Literal(e) => visitor.visit_literal(e),
            Expr::Unary(e) => visitor.visit_unary(e),
            Expr::Variable(e) => visitor.visit_variable(e),
            Expr::Assign(e) => visitor.visit_assign(e),
        }
    }

    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary(e) => e.left.span().join(&e.right.span()),
            Expr::Grouping(e) => e.0.span(),
            Expr::Literal(e) => e.token.span.clone(),
            Expr::Unary(e) => e.op.span.join(&e.right.span()),
            Expr::Variable(e) => e.name.span.clone(),
            Expr::Assign(e) => e.name.span.join(&e.value.span()),
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
        self.accept(&mut StdPrinter { fmt: f })
    }
}

struct StdPrinter<'a, 'f> {
    fmt: &'a mut fmt::Formatter<'f>,
}

impl ExprVisitor for StdPrinter<'_, '_> {
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

    fn visit_variable(&mut self, expr: &ExprVariable) -> Self::T {
        write!(self.fmt, "{}", expr.name.ty)
    }

    fn visit_assign(&mut self, expr: &ExprAssign) -> Self::T {
        write!(self.fmt, "{} = ", expr.name.ty)?;
        expr.value.accept(self)
    }
}

pub struct AstPrinter<'a, 'f> {
    pub fmt: &'a mut fmt::Formatter<'f>,
}

impl ExprVisitor for AstPrinter<'_, '_> {
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

    fn visit_variable(&mut self, expr: &ExprVariable) -> Self::T {
        write!(self.fmt, "{}", expr.name.ty)
    }

    fn visit_assign(&mut self, expr: &ExprAssign) -> Self::T {
        write!(self.fmt, "(= {}", expr.name.ty)?;
        expr.value.accept(self)?;
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

impl From<ExprVariable> for Expr {
    fn from(value: ExprVariable) -> Self {
        Self::Variable(value)
    }
}

impl From<ExprAssign> for Expr {
    fn from(value: ExprAssign) -> Self {
        Self::Assign(value)
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
