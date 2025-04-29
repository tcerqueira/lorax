use super::expr::*;
use super::stmt::*;

pub trait ExprVisitor {
    type T;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T;
    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T;
    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T;
    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T;
}

pub trait StmtVisitor {
    type T;

    fn visit_print(&mut self, stmt: &StmtPrint) -> Self::T;
    fn visit_expression(&mut self, stmt: &StmtExpression) -> Self::T;
}
