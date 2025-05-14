use super::{expr::Expr, visitor::StmtVisitor};
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(StmtPrint),
    Expression(StmtExpression),
    Var(StmtVar),
    Block(StmtBlock),
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

#[derive(Debug, Clone)]
pub struct StmtBlock {
    pub statements: Vec<Stmt>,
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut impl StmtVisitor<T = R>) -> R {
        match self {
            Stmt::Print(s) => visitor.visit_print(s),
            Stmt::Expression(s) => visitor.visit_expression(s),
            Stmt::Var(s) => visitor.visit_var(s),
            Stmt::Block(s) => visitor.visit_block(s),
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

impl From<StmtBlock> for Stmt {
    fn from(value: StmtBlock) -> Self {
        Stmt::Block(value)
    }
}
