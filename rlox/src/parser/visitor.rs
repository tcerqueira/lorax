use super::expr::*;

pub trait Visitor {
    type T;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T;
    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T;
    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T;
    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T;
}
