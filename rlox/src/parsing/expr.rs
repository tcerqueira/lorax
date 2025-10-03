use std::fmt::{self, Debug, Display};

use derive_more::From;

use super::visitor::ExprVisitor;
use crate::{
    parsing::ast::{AstNode, AstRef, ExprId, ExprRef},
    report::{Span, Spanned},
    runtime::object::Object,
    tokens::Token,
};

#[derive(Debug, Clone, PartialEq, From)]
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

impl Spanned for ExprRef<'_> {
    fn span(&self) -> Span {
        match **self {
            Expr::Binary(_) => self.cast::<ExprBinary>().span(),
            Expr::Call(_) => self.cast::<ExprCall>().span(),
            Expr::Grouping(_) => self.cast::<ExprGrouping>().span(),
            Expr::Literal(_) => self.cast::<ExprLiteral>().span(),
            Expr::Unary(_) => self.cast::<ExprUnary>().span(),
            Expr::Variable(_) => self.cast::<ExprVariable>().span(),
            Expr::Assign(_) => self.cast::<ExprAssign>().span(),
            Expr::Logical(_) => self.cast::<ExprLogical>().span(),
        }
    }
}

macro_rules! impl_expr_node {
    ($variant:path, $type:ident) => {
        $crate::impl_ast_node!(Expr, $variant, $type);
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

impl Spanned for AstRef<'_, ExprBinary> {
    fn span(&self) -> Span {
        let left = ExprRef::new(self.arena(), self.left);
        let right = ExprRef::new(self.arena(), self.right);
        left.span().join(&right.span())
    }
}

impl Spanned for AstRef<'_, ExprUnary> {
    fn span(&self) -> Span {
        let right = ExprRef::new(self.arena(), self.right);
        self.op.span.join(&right.span())
    }
}

impl Spanned for AstRef<'_, ExprCall> {
    fn span(&self) -> Span {
        let callee = ExprRef::new(self.arena(), self.callee);
        callee.span().join(&self.r_paren.span)
    }
}

impl Spanned for AstRef<'_, ExprGrouping> {
    fn span(&self) -> Span {
        ExprRef::new(self.arena(), self.0).span()
    }
}

impl Spanned for AstRef<'_, ExprLiteral> {
    fn span(&self) -> Span {
        self.token.span.clone()
    }
}

impl Spanned for AstRef<'_, ExprVariable> {
    fn span(&self) -> Span {
        self.name.span.clone()
    }
}

impl Spanned for AstRef<'_, ExprAssign> {
    fn span(&self) -> Span {
        let value = ExprRef::new(self.arena(), self.value);
        self.name.span.join(&value.span())
    }
}

impl Spanned for AstRef<'_, ExprLogical> {
    fn span(&self) -> Span {
        let left = ExprRef::new(self.arena(), self.left);
        let right = ExprRef::new(self.arena(), self.right);
        left.span().join(&right.span())
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
