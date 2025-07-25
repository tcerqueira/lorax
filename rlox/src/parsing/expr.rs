use std::fmt::{self, Debug, Display};

use super::visitor::ExprVisitor;
use crate::{
    parsing::ast::{AstNode, AstRef, ExprId, ExprRef},
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
    pub left: ExprId,
    pub right: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub callee: ExprId,
    pub r_paren: Token,
    pub args: Vec<ExprId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprGrouping(pub ExprId);

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
    pub right: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprVariable {
    pub name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign {
    pub name: Token,
    pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprLogical {
    pub op: Token,
    pub left: ExprId,
    pub right: ExprId,
}

impl ExprRef<'_> {
    #[cfg(test)]
    pub fn polish_notation(&self) -> String {
        struct PolishNotation<'a>(ExprRef<'a>);
        impl Display for PolishNotation<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.accept(&mut AstPrinter { fmt: f })
            }
        }

        PolishNotation(*self).to_string()
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

            fn deref_node(node: AstRef<'_, Self>) -> &Self {
                match &node.arena()[node.id()] {
                    $variant(expr) => expr,
                    _ => panic!(
                        "failed to unwrap {} on {}",
                        std::any::type_name::<Self>(),
                        std::any::type_name::<Self::NodeType>()
                    ),
                }
            }
        }
    };
}

impl AstNode for Expr {
    type NodeType = Expr;

    fn deref_node(node: AstRef<'_, Self>) -> &Self {
        &node.arena()[node.id()]
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

impl Display for ExprRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.accept(&mut StdPrinter { fmt: f })
    }
}

struct StdPrinter<'a, 'f> {
    fmt: &'a mut fmt::Formatter<'f>,
}

impl ExprVisitor for &mut StdPrinter<'_, '_> {
    type T = fmt::Result;

    fn visit_binary(self, expr: AstRef<ExprBinary>) -> Self::T {
        let arena = expr.arena();
        let left = arena.expr_ref(expr.left);
        let right = arena.expr_ref(expr.right);

        left.accept(&mut *self)?;
        write!(self.fmt, " {} ", expr.op.ty)?;
        right.accept(self)
    }

    fn visit_call(self, expr: AstRef<ExprCall>) -> Self::T {
        let arena = expr.arena();
        let callee = arena.expr_ref(expr.callee);

        callee.accept(&mut *self)?;
        write!(self.fmt, "(")?;
        let mut iter = expr.args.iter().peekable();
        while let Some(arg) = iter.next().map(|&a| arena.expr_ref(a)) {
            arg.accept(&mut *self)?;
            if iter.peek().is_some() {
                write!(self.fmt, ", ")?;
            }
        }
        write!(self.fmt, ")")
    }

    fn visit_grouping(self, expr: AstRef<ExprGrouping>) -> Self::T {
        let arena = expr.arena();

        write!(self.fmt, "(")?;
        arena.expr_ref(expr.0).accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(self, expr: AstRef<ExprLiteral>) -> Self::T {
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(self, expr: AstRef<ExprUnary>) -> Self::T {
        let arena = expr.arena();

        write!(self.fmt, "{}", expr.op.ty)?;
        arena.expr_ref(expr.right).accept(self)
    }

    fn visit_variable(self, expr: AstRef<ExprVariable>) -> Self::T {
        write!(self.fmt, "{}", expr.name.ty)
    }

    fn visit_assign(self, expr: AstRef<ExprAssign>) -> Self::T {
        let arena = expr.arena();

        write!(self.fmt, "{} = ", expr.name.ty)?;
        arena.expr_ref(expr.value).accept(self)
    }

    fn visit_logical(self, expr: AstRef<ExprLogical>) -> Self::T {
        let arena = expr.arena();
        let left = arena.expr_ref(expr.left);
        let right = arena.expr_ref(expr.right);

        left.accept(&mut *self)?;
        write!(self.fmt, " {} ", expr.op.ty)?;
        right.accept(self)
    }
}

pub struct AstPrinter<'a, 'f> {
    pub fmt: &'a mut fmt::Formatter<'f>,
}

impl ExprVisitor for &mut AstPrinter<'_, '_> {
    type T = fmt::Result;

    fn visit_binary(self, expr: AstRef<ExprBinary>) -> Self::T {
        let arena = expr.arena();
        let left = arena.expr_ref(expr.left);
        let right = arena.expr_ref(expr.right);

        write!(self.fmt, "({} ", expr.op.ty)?;
        left.accept(&mut *self)?;
        write!(self.fmt, " ")?;
        right.accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_call(self, expr: AstRef<ExprCall>) -> Self::T {
        let arena = expr.arena();
        let callee = arena.expr_ref(expr.callee);

        write!(self.fmt, "(call ")?;
        callee.accept(&mut *self)?;
        if !expr.args.is_empty() {
            write!(self.fmt, " ")?;
        }
        let mut iter = expr.args.iter().peekable();
        while let Some(arg) = iter.next().map(|&a| arena.expr_ref(a)) {
            arg.accept(&mut *self)?;
            if iter.peek().is_some() {
                write!(self.fmt, ", ")?;
            }
        }
        write!(self.fmt, ")")
    }

    fn visit_grouping(self, expr: AstRef<ExprGrouping>) -> Self::T {
        let arena = expr.arena();

        write!(self.fmt, "(group ")?;
        arena.expr_ref(expr.0).accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_literal(self, expr: AstRef<ExprLiteral>) -> Self::T {
        write!(self.fmt, "{}", expr.token.ty)
    }

    fn visit_unary(self, expr: AstRef<ExprUnary>) -> Self::T {
        let arena = expr.arena();

        write!(self.fmt, "({} ", expr.op.ty)?;
        arena.expr_ref(expr.right).accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_variable(self, expr: AstRef<ExprVariable>) -> Self::T {
        write!(self.fmt, "{}", expr.name.ty)
    }

    fn visit_assign(self, expr: AstRef<ExprAssign>) -> Self::T {
        let arena = expr.arena();

        write!(self.fmt, "(= {}", expr.name.ty)?;
        arena.expr_ref(expr.value).accept(&mut *self)?;
        write!(self.fmt, ")")
    }

    fn visit_logical(self, expr: AstRef<ExprLogical>) -> Self::T {
        let arena = expr.arena();
        let left = arena.expr_ref(expr.left);
        let right = arena.expr_ref(expr.right);

        write!(self.fmt, "({} ", expr.op.ty)?;
        left.accept(&mut *self)?;
        write!(self.fmt, " ")?;
        right.accept(&mut *self)?;
        write!(self.fmt, ")")
    }
}

impl Spanned for ExprBinary {
    fn span(&self) -> Span {
        // self.left.span().join(&self.right.span())
        Span::default()
    }
}

impl Spanned for ExprUnary {
    fn span(&self) -> Span {
        // self.op.span.join(&self.right.span())
        self.op.span.clone()
    }
}

impl Spanned for ExprCall {
    fn span(&self) -> Span {
        // self.callee.span().join(&self.r_paren.span)
        Span::default()
    }
}

impl Spanned for ExprGrouping {
    fn span(&self) -> Span {
        // self.0.span()
        Span::default()
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
        // self.name.span.join(&self.value.span())
        self.name.span.clone()
    }
}

impl Spanned for ExprLogical {
    fn span(&self) -> Span {
        // self.left.span().join(&self.right.span())
        Span::default()
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
    use crate::{parsing::ast::AstArena, tok};

    #[test]
    fn test_printer() {
        let mut arena = AstArena::default();
        let unary_left = arena
            .alloc_expr(
                ExprLiteral {
                    token: (tok![n:123]),
                    literal: Object::nil(),
                }
                .into(),
            )
            .id();
        let left = arena
            .alloc_expr(
                ExprUnary {
                    op: tok![-],
                    right: unary_left,
                }
                .into(),
            )
            .id();
        let literal_right = arena
            .alloc_expr(
                ExprLiteral {
                    token: (tok![n:45.67]),
                    literal: Object::nil(),
                }
                .into(),
            )
            .id();
        let right = arena.alloc_expr(ExprGrouping(literal_right).into()).id();

        let expr = arena.alloc_expr(
            ExprBinary {
                op: tok![*],
                left,
                right,
            }
            .into(),
        );

        assert_eq!("(* (- 123) (group 45.67))", expr.polish_notation());
    }
}
