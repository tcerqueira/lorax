use std::fmt::{self, Debug, Display};

use super::visitor::ExprVisitor;
use crate::{
    parsing::ast::AstNode,
    report::{Span, Spanned},
    runtime::object::Object,
    tokens::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(ExprBinary),
    Call(ExprCall),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
    Variable(ExprVariable),
    Assign(ExprAssign),
    Logical(ExprLogical),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprBinary {
    pub op: Token,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub callee: Box<Expr>,
    pub r_paren: Token,
    pub args: Vec<Expr>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ExprLogical {
    pub op: Token,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Expr {
    pub fn accept<R>(&self, visitor: impl ExprVisitor<T = R>) -> R {
        match self {
            Expr::Binary(e) => visitor.visit_binary(e),
            Expr::Call(e) => visitor.visit_call(e),
            Expr::Grouping(e) => visitor.visit_grouping(e),
            Expr::Literal(e) => visitor.visit_literal(e),
            Expr::Unary(e) => visitor.visit_unary(e),
            Expr::Variable(e) => visitor.visit_variable(e),
            Expr::Assign(e) => visitor.visit_assign(e),
            Expr::Logical(e) => visitor.visit_logical(e),
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

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Binary(e) => e.span(),
            Expr::Call(e) => e.span(),
            Expr::Grouping(e) => e.span(),
            Expr::Literal(e) => e.span(),
            Expr::Unary(e) => e.span(),
            Expr::Variable(e) => e.span(),
            Expr::Assign(e) => e.span(),
            Expr::Logical(e) => e.span(),
        }
    }
}

macro_rules! impl_expr_node {
    ($variant:path, $type:ident) => {
        impl $crate::parsing::ast::AstNode for $type {
            type NodeType = Expr;

            fn try_as_variant(expr: &$crate::parsing::expr::Expr) -> Option<&Self> {
                match expr {
                    $variant(expr) => Some(expr),
                    _ => None,
                }
            }
        }
    };
}

impl AstNode for Expr {
    type NodeType = Expr;

    fn try_as_variant(node: &Self::NodeType) -> Option<&Self> {
        Some(node)
    }
}

impl_expr_node!(Expr::Binary, ExprBinary);
impl_expr_node!(Expr::Call, ExprCall);
impl_expr_node!(Expr::Grouping, ExprGrouping);
impl_expr_node!(Expr::Literal, ExprLiteral);
impl_expr_node!(Expr::Unary, ExprUnary);
impl_expr_node!(Expr::Variable, ExprVariable);
impl_expr_node!(Expr::Assign, ExprAssign);
impl_expr_node!(Expr::Logical, ExprLogical);

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.accept(&mut StdPrinter { fmt: f })
    }
}

struct StdPrinter<'a, 'f> {
    fmt: &'a mut fmt::Formatter<'f>,
}

impl ExprVisitor for &mut StdPrinter<'_, '_> {
    type T = fmt::Result;

    fn visit_binary(self, expr: &ExprBinary) -> Self::T {
        expr.left.accept(&mut *self)?;
        write!(self.fmt, " {} ", expr.op.ty)?;
        expr.right.accept(self)
    }

    fn visit_call(self, expr: &ExprCall) -> Self::T {
        expr.callee.accept(&mut *self)?;
        write!(self.fmt, "(")?;
        let mut iter = expr.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.accept(&mut *self)?;
            if iter.peek().is_some() {
                write!(self.fmt, ", ")?;
            }
        }
        write!(self.fmt, ")")
    }

    fn visit_grouping(self, expr: &ExprGrouping) -> Self::T {
        write!(self.fmt, "(")?;
        expr.0.accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(self, expr: &ExprLiteral) -> Self::T {
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(self, expr: &ExprUnary) -> Self::T {
        write!(self.fmt, "{}", expr.op.ty)?;
        expr.right.accept(self)
    }

    fn visit_variable(self, expr: &ExprVariable) -> Self::T {
        write!(self.fmt, "{}", expr.name.ty)
    }

    fn visit_assign(self, expr: &ExprAssign) -> Self::T {
        write!(self.fmt, "{} = ", expr.name.ty)?;
        expr.value.accept(self)
    }

    fn visit_logical(self, expr: &ExprLogical) -> Self::T {
        expr.left.accept(&mut *self)?;
        write!(self.fmt, " {} ", expr.op.ty)?;
        expr.right.accept(self)
    }
}

pub struct AstPrinter<'a, 'f> {
    pub fmt: &'a mut fmt::Formatter<'f>,
}

impl ExprVisitor for &mut AstPrinter<'_, '_> {
    type T = fmt::Result;

    fn visit_binary(self, expr: &ExprBinary) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.left.accept(&mut *self)?;
        write!(self.fmt, " ")?;
        expr.right.accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_call(self, expr: &ExprCall) -> Self::T {
        write!(self.fmt, "(call ")?;
        expr.callee.accept(&mut *self)?;
        if !expr.args.is_empty() {
            write!(self.fmt, " ")?;
        }
        let mut iter = expr.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.accept(&mut *self)?;
            if iter.peek().is_some() {
                write!(self.fmt, ", ")?;
            }
        }
        write!(self.fmt, ")")
    }

    fn visit_grouping(self, expr: &ExprGrouping) -> Self::T {
        write!(self.fmt, "(group ")?;
        expr.0.accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(self, expr: &ExprLiteral) -> Self::T {
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(self, expr: &ExprUnary) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.right.accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_variable(self, expr: &ExprVariable) -> Self::T {
        write!(self.fmt, "{}", expr.name.ty)
    }

    fn visit_assign(self, expr: &ExprAssign) -> Self::T {
        write!(self.fmt, "(= {}", expr.name.ty)?;
        expr.value.accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_logical(self, expr: &ExprLogical) -> Self::T {
        write!(self.fmt, "({} ", expr.op.ty)?;
        expr.left.accept(&mut *self)?;
        write!(self.fmt, " ")?;
        expr.right.accept(&mut *self)?;
        write!(self.fmt, ")")
    }
}

impl Spanned for ExprBinary {
    fn span(&self) -> Span {
        self.left.span().join(&self.right.span())
    }
}

impl Spanned for ExprUnary {
    fn span(&self) -> Span {
        self.op.span.join(&self.right.span())
    }
}

impl Spanned for ExprCall {
    fn span(&self) -> Span {
        self.callee.span().join(&self.r_paren.span)
    }
}

impl Spanned for ExprGrouping {
    fn span(&self) -> Span {
        self.0.span()
    }
}

impl Spanned for ExprLiteral {
    fn span(&self) -> Span {
        self.token.span.clone()
    }
}

impl Spanned for ExprVariable {
    fn span(&self) -> Span {
        self.name.span.clone()
    }
}

impl Spanned for ExprAssign {
    fn span(&self) -> Span {
        self.name.span.join(&self.value.span())
    }
}

impl Spanned for ExprLogical {
    fn span(&self) -> Span {
        self.left.span().join(&self.right.span())
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

impl From<ExprCall> for Expr {
    fn from(value: ExprCall) -> Self {
        Self::Call(value)
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

impl From<ExprLogical> for Expr {
    fn from(value: ExprLogical) -> Self {
        Self::Logical(value)
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
