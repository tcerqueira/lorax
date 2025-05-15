use super::{expr::*, stmt::*};

pub trait ExprVisitor {
    type T;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T;
    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T;
    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T;
    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T;
    fn visit_variable(&mut self, expr: &ExprVariable) -> Self::T;
    fn visit_assign(&mut self, expr: &ExprAssign) -> Self::T;
    fn visit_logical(&mut self, expr: &ExprLogical) -> Self::T;
}

pub trait StmtVisitor {
    type T;

    fn visit_print(&mut self, stmt: &StmtPrint) -> Self::T;
    fn visit_expression(&mut self, stmt: &StmtExpression) -> Self::T;
    fn visit_var(&mut self, stmt: &StmtVar) -> Self::T;
    fn visit_block(&mut self, stmt: &StmtBlock) -> Self::T;
    fn visit_if(&mut self, stmt: &StmtIf) -> Self::T;
    fn visit_while(&mut self, stmt: &StmtWhile) -> Self::T;
}
