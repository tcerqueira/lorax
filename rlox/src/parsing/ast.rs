#![allow(dead_code)]
use std::ops::{Deref, Index, IndexMut};

use slotmap::{Key, SlotMap, new_key_type};

use crate::parsing::{expr::Expr, stmt::Stmt};

new_key_type! { pub struct ExprId; }
new_key_type! { pub struct StmtId; }

#[derive(Debug, Default)]
pub struct AstArena {
    expressions: SlotMap<ExprId, Expr>,
    statements: SlotMap<StmtId, Stmt>,
}

impl AstArena {
    pub fn alloc_expr(&mut self, expr: Expr) -> ExprRef<'_> {
        let id = self.expressions.insert(expr);
        AstRef::new(self, id)
    }

    pub fn alloc_stmt(&mut self, stmt: Stmt) -> StmtRef<'_> {
        let id = self.statements.insert(stmt);
        AstRef::new(self, id)
    }

    pub fn expr_ref(&self, id: ExprId) -> AstRef<'_, Expr> {
        AstRef::new(self, id)
    }

    pub fn stmt_ref(&self, id: StmtId) -> AstRef<'_, Stmt> {
        AstRef::new(self, id)
    }
}

impl Index<ExprId> for AstArena {
    type Output = Expr;

    fn index(&self, index: ExprId) -> &Self::Output {
        &self.expressions[index]
    }
}

impl IndexMut<ExprId> for AstArena {
    fn index_mut(&mut self, index: ExprId) -> &mut Self::Output {
        &mut self.expressions[index]
    }
}

impl Index<StmtId> for AstArena {
    type Output = Stmt;

    fn index(&self, index: StmtId) -> &Self::Output {
        &self.statements[index]
    }
}

impl IndexMut<StmtId> for AstArena {
    fn index_mut(&mut self, index: StmtId) -> &mut Self::Output {
        &mut self.statements[index]
    }
}

pub trait IdType {
    type Id: Key;
}

impl IdType for Expr {
    type Id = ExprId;
}

impl IdType for Stmt {
    type Id = StmtId;
}

pub trait AstNode {
    type NodeType: IdType;

    fn deref_node(node: AstRef<'_, Self>) -> &Self;
}

pub type ExprRef<'a> = AstRef<'a, Expr>;
pub type StmtRef<'a> = AstRef<'a, Stmt>;

#[derive(Debug)]
pub struct AstRef<'a, N: AstNode + ?Sized> {
    ast_arena: &'a AstArena,
    id: <N::NodeType as IdType>::Id,
}

impl<'a, N: AstNode> AstRef<'a, N> {
    pub fn new(ast_arena: &'a AstArena, id: <N::NodeType as IdType>::Id) -> Self {
        Self { ast_arena, id }
    }

    pub fn arena(&self) -> &'a AstArena {
        self.ast_arena
    }

    pub fn id(&self) -> <N::NodeType as IdType>::Id {
        self.id
    }
}

impl<'a, N: AstNode<NodeType = N> + IdType> AstRef<'a, N> {
    pub fn cast<T>(&self) -> AstRef<'a, T>
    where
        T: AstNode<NodeType = N>,
    {
        AstRef::<T>::new(self.ast_arena, self.id)
    }
}

impl<'a, N: AstNode> Deref for AstRef<'a, N> {
    type Target = N;

    fn deref(&self) -> &Self::Target {
        N::deref_node(*self)
    }
}

impl<'a, T, N: AstNode> AsRef<T> for AstRef<'a, N>
where
    T: ?Sized,
    <AstRef<'a, N> as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<N: AstNode> Clone for AstRef<'_, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: AstNode> Copy for AstRef<'_, N> {}

#[macro_export]
macro_rules! impl_ast_node {
    ($node:ident, $variant:path, $type:ident) => {
        impl $crate::parsing::ast::AstNode for $type {
            type NodeType = $node;

            fn deref_node(node: AstRef<'_, Self>) -> &Self {
                match &node.arena()[node.id()] {
                    $variant(node) => node,
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
