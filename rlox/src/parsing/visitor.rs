use crate::parsing::ast::AstRef;

use super::{expr::*, stmt::*};

pub trait ExprVisitor {
    type T;

    fn visit_binary(self, expr: AstRef<ExprBinary>) -> Self::T;
    fn visit_call(self, expr: AstRef<ExprCall>) -> Self::T;
    fn visit_grouping(self, expr: AstRef<ExprGrouping>) -> Self::T;
    fn visit_literal(self, expr: AstRef<ExprLiteral>) -> Self::T;
    fn visit_unary(self, expr: AstRef<ExprUnary>) -> Self::T;
    fn visit_variable(self, expr: AstRef<ExprVariable>) -> Self::T;
    fn visit_assign(self, expr: AstRef<ExprAssign>) -> Self::T;
    fn visit_logical(self, expr: AstRef<ExprLogical>) -> Self::T;
}

pub trait StmtVisitor {
    type T;

    fn visit_print(self, stmt: AstRef<StmtPrint>) -> Self::T;
    fn visit_expression(self, stmt: AstRef<StmtExpression>) -> Self::T;
    fn visit_var(self, stmt: AstRef<StmtVar>) -> Self::T;
    fn visit_block(self, stmt: AstRef<StmtBlock>) -> Self::T;
    fn visit_if(self, stmt: AstRef<StmtIf>) -> Self::T;
    fn visit_return(self, stmt: AstRef<StmtReturn>) -> Self::T;
    fn visit_while(self, stmt: AstRef<StmtWhile>) -> Self::T;
    fn visit_function(self, stmt: AstRef<StmtFunction>) -> Self::T;
}

impl<'a> AstRef<'a, Expr> {
    pub fn accept<R>(&self, visitor: impl ExprVisitor<T = R>) -> R {
        match **self {
            Expr::Binary(_) => visitor.visit_binary(self.cast()),
            Expr::Call(_) => visitor.visit_call(self.cast()),
            Expr::Grouping(_) => visitor.visit_grouping(self.cast()),
            Expr::Literal(_) => visitor.visit_literal(self.cast()),
            Expr::Unary(_) => visitor.visit_unary(self.cast()),
            Expr::Variable(_) => visitor.visit_variable(self.cast()),
            Expr::Assign(_) => visitor.visit_assign(self.cast()),
            Expr::Logical(_) => visitor.visit_logical(self.cast()),
        }
    }
}

impl<'a> AstRef<'a, Stmt> {
    pub fn accept<R>(&self, visitor: impl StmtVisitor<T = R>) -> R {
        match **self {
            Stmt::Print(_) => visitor.visit_print(self.cast()),
            Stmt::Expression(_) => visitor.visit_expression(self.cast()),
            Stmt::Var(_) => visitor.visit_var(self.cast()),
            Stmt::Block(_) => visitor.visit_block(self.cast()),
            Stmt::If(_) => visitor.visit_if(self.cast()),
            Stmt::Return(_) => visitor.visit_return(self.cast()),
            Stmt::While(_) => visitor.visit_while(self.cast()),
            Stmt::Function(_) => visitor.visit_function(self.cast()),
        }
    }
}
