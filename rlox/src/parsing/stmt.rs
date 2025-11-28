use derive_more::From;
use rlox_lexer::tokens::Token;

use crate::parsing::ast::{AstNode, AstRef, ExprId, StmtId};

#[derive(Debug, Clone, From)]
pub enum Stmt {
    Print(StmtPrint),
    Expression(StmtExpression),
    Var(StmtVar),
    Block(StmtBlock),
    If(StmtIf),
    Return(StmtReturn),
    While(StmtWhile),
    Function(StmtFunction),
}

#[derive(Debug, Clone)]
pub struct StmtPrint {
    pub print_token: Token,
    pub expr: ExprId,
}

#[derive(Debug, Clone)]
pub struct StmtExpression {
    pub expr: ExprId,
}

#[derive(Debug, Clone)]
pub struct StmtVar {
    pub ident: Token,
    pub initializer: Option<ExprId>,
}

#[derive(Debug, Clone)]
pub struct StmtBlock {
    pub statements: Vec<StmtId>,
}

#[derive(Debug, Clone)]
pub struct StmtIf {
    pub condition: ExprId,
    pub then_branch: StmtId,
    pub else_branch: Option<StmtId>,
}

#[derive(Debug, Clone)]
pub struct StmtReturn {
    pub return_token: Token,
    pub expr: Option<ExprId>,
}

#[derive(Debug, Clone)]
pub struct StmtWhile {
    pub condition: ExprId,
    pub body: StmtId,
}

#[derive(Debug, Clone)]
pub struct StmtFunction {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<StmtId>,
}

macro_rules! impl_stmt_node {
    ($variant:path, $type:ident) => {
        $crate::impl_ast_node!(Stmt, $variant, $type);
    };
}

impl AstNode for Stmt {
    type NodeType = Stmt;

    fn deref_node(node: AstRef<'_, Self>) -> &Self {
        &node.arena()[node.id()]
    }
}

impl_stmt_node!(Stmt::Print, StmtPrint);
impl_stmt_node!(Stmt::Expression, StmtExpression);
impl_stmt_node!(Stmt::Var, StmtVar);
impl_stmt_node!(Stmt::Block, StmtBlock);
impl_stmt_node!(Stmt::If, StmtIf);
impl_stmt_node!(Stmt::Return, StmtReturn);
impl_stmt_node!(Stmt::While, StmtWhile);
impl_stmt_node!(Stmt::Function, StmtFunction);

impl From<ExprId> for StmtExpression {
    fn from(expr: ExprId) -> Self {
        StmtExpression { expr }
    }
}

impl From<ExprId> for Stmt {
    fn from(expr: ExprId) -> Self {
        Stmt::Expression(expr.into())
    }
}
