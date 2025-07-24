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

    pub fn alloc_stmt(&mut self, stmt: Stmt) -> StmtId {
        self.statements.insert(stmt)
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

trait IdType {
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

    fn try_as_variant(node: &Self::NodeType) -> Option<&Self>;

    fn as_variant(node: &Self::NodeType) -> &Self {
        Self::try_as_variant(node).unwrap_or_else(|| {
            panic!(
                "failed to unwrap {} on {}",
                std::any::type_name::<Self>(),
                std::any::type_name::<Self::NodeType>()
            )
        })
    }
}

pub type ExprRef<'a> = AstRef<'a, Expr>;
pub type StmtRef<'a> = AstRef<'a, Stmt>;

pub struct AstRef<'a, E: AstNode> {
    ast_arena: &'a AstArena,
    id: <E::NodeType as IdType>::Id,
}

impl<'a, E: AstNode> AstRef<'a, E> {
    pub fn new(ast_arena: &'a AstArena, id: <E::NodeType as IdType>::Id) -> Self {
        Self { ast_arena, id }
    }

    pub fn arena(&self) -> &'a AstArena {
        self.ast_arena
    }

    pub fn id(&self) -> <E::NodeType as IdType>::Id {
        self.id
    }
}

impl<'a, E: AstNode<NodeType = Expr>> AstRef<'a, E> {
    pub fn as_variant(&self) -> &'a E {
        E::as_variant(&self.ast_arena[self.id])
    }
}

impl<'a, E: AstNode<NodeType = Expr>> Deref for AstRef<'a, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.as_variant()
    }
}

impl<'a, T, E: AstNode<NodeType = Expr>> AsRef<T> for AstRef<'a, E>
where
    T: ?Sized,
    <AstRef<'a, E> as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}
