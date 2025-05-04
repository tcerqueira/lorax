use super::{expr::Expr, visitor::StmtVisitor};
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(StmtPrint),
    Expression(StmtExpression),
    Var(StmtVar),
}

#[derive(Debug, Clone)]
pub struct StmtPrint {
    #[expect(dead_code)]
    pub print_token: Token,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct StmtExpression {
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct StmtVar {
    pub ident: Token,
    pub initializer: Option<Expr>,
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut impl StmtVisitor<T = R>) -> R {
        match self {
            Stmt::Print(stmt_print) => visitor.visit_print(stmt_print),
            Stmt::Expression(stmt_expression) => visitor.visit_expression(stmt_expression),
            Stmt::Var(stmt_var) => visitor.visit_var(stmt_var),
        }
    }
}

impl From<StmtPrint> for Stmt {
    fn from(value: StmtPrint) -> Self {
        Stmt::Print(value)
    }
}

impl From<StmtExpression> for Stmt {
    fn from(value: StmtExpression) -> Self {
        Stmt::Expression(value)
    }
}

impl From<StmtVar> for Stmt {
    fn from(value: StmtVar) -> Self {
        Stmt::Var(value)
    }
}
