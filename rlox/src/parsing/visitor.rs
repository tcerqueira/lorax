use super::{expr::*, stmt::*};

pub trait ExprVisitor {
    type T;

    fn visit_binary(self, expr: &ExprBinary) -> Self::T;
    fn visit_call(self, expr: &ExprCall) -> Self::T;
    fn visit_grouping(self, expr: &ExprGrouping) -> Self::T;
    fn visit_literal(self, expr: &ExprLiteral) -> Self::T;
    fn visit_unary(self, expr: &ExprUnary) -> Self::T;
    fn visit_variable(self, expr: &ExprVariable) -> Self::T;
    fn visit_assign(self, expr: &ExprAssign) -> Self::T;
    fn visit_logical(self, expr: &ExprLogical) -> Self::T;
}

pub trait StmtVisitor {
    type T;

    fn visit_print(self, stmt: &StmtPrint) -> Self::T;
    fn visit_expression(self, stmt: &StmtExpression) -> Self::T;
    fn visit_var(self, stmt: &StmtVar) -> Self::T;
    fn visit_block(self, stmt: &StmtBlock) -> Self::T;
    fn visit_if(self, stmt: &StmtIf) -> Self::T;
    fn visit_while(self, stmt: &StmtWhile) -> Self::T;
    fn visit_function(self, stmt: &StmtFunction) -> Self::T;
}
