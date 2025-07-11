use crate::{
    parsing::ast::{AstNode, AstRef, ExprId, StmtId},
    tokens::Token,
};

#[derive(Debug, Clone)]
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
    #[expect(dead_code)]
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
    #[expect(dead_code)]
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
        impl $crate::parsing::ast::AstNode for $type {
            type NodeType = Stmt;

            fn deref_node(node: $crate::parsing::ast::AstRef<'_, Self>) -> &Self {
                match &node.arena()[node.id()] {
                    $variant(stmt) => stmt,
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

impl From<StmtIf> for Stmt {
    fn from(value: StmtIf) -> Self {
        Stmt::If(value)
    }
}

impl From<StmtReturn> for Stmt {
    fn from(value: StmtReturn) -> Self {
        Stmt::Return(value)
    }
}

impl From<StmtWhile> for Stmt {
    fn from(value: StmtWhile) -> Self {
        Stmt::While(value)
    }
}

impl From<StmtFunction> for Stmt {
    fn from(value: StmtFunction) -> Self {
        Stmt::Function(value)
    }
}
