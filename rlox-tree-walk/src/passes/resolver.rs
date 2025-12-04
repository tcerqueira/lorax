use std::collections::HashSet;

use crate::{
    parsing::{
        ast::{AstArena, AstRef, ExprId, ExprRef, StmtId, StmtRef},
        expr::*,
        stmt::*,
        visitor::{ExprVisitor, StmtVisitor},
    },
    runtime::Interpreter,
};

pub struct Resolver<'i, 'a> {
    interpreter: &'i mut Interpreter,
    ast_arena: &'a AstArena,
    scope_stack: Vec<HashSet<Box<str>>>,
}

impl<'i, 'a> Resolver<'i, 'a> {
    pub fn new(interpreter: &'i mut Interpreter, ast_arena: &'a AstArena) -> Self {
        Self {
            interpreter,
            ast_arena,
            scope_stack: vec![HashSet::new()],
        }
    }

    fn resolve_expr(&mut self, expr: ExprRef) {
        expr.accept(self)
    }

    fn resolve_stmt(&mut self, stmt: StmtRef) {
        stmt.accept(self)
    }

    pub fn resolve(&mut self, stmts: &[StmtId]) {
        for stmt in stmts.iter().map(|&id| self.ast_arena.stmt_ref(id)) {
            stmt.accept(&mut *self);
        }
    }

    fn resolve_local(&mut self, expr_id: ExprId, name: &str) {
        let depth = self
            .scope_stack
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, scope)| scope.contains(name))
            .map(|(i, _s)| i);

        if let Some(depth) = depth {
            self.interpreter
                .resolve_var(expr_id, self.scope_stack.len() - 1 - depth);
        }
    }

    fn resolve_fn(&mut self, stmt_fn: AstRef<StmtFunction>) {
        self.begin_scope();
        for stmt in &stmt_fn.params {
            self.define(stmt.as_str().into());
        }
        self.resolve(&stmt_fn.body);
        self.end_scope();
    }

    fn define(&mut self, name: Box<str>) {
        self.scope_stack
            .last_mut()
            .expect("at least the global scope is present")
            .insert(name);
    }

    fn begin_scope(&mut self) {
        self.scope_stack.push(HashSet::new());
    }

    fn end_scope(&mut self) {
        self.scope_stack.pop();
    }
}

impl StmtVisitor for &mut Resolver<'_, '_> {
    type T = ();

    fn visit_print(self, stmt: AstRef<StmtPrint>) {
        self.resolve_expr(self.ast_arena.expr_ref(stmt.expr))
    }

    fn visit_expression(self, stmt: AstRef<StmtExpression>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(stmt.expr))
    }

    fn visit_var(self, stmt: AstRef<StmtVar>) -> Self::T {
        if let Some(initializer) = stmt.initializer {
            let initializer = self.ast_arena.expr_ref(initializer);
            self.resolve_expr(initializer);
        }
        self.define(stmt.ident.as_str().into());
    }

    fn visit_block(self, stmt: AstRef<StmtBlock>) -> Self::T {
        self.begin_scope();
        self.resolve(&stmt.statements);
        self.end_scope();
    }

    fn visit_if(self, stmt: AstRef<StmtIf>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(stmt.condition));
        self.resolve_stmt(self.ast_arena.stmt_ref(stmt.then_branch));
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(self.ast_arena.stmt_ref(*else_branch));
        }
    }

    fn visit_return(self, stmt: AstRef<StmtReturn>) -> Self::T {
        if let Some(expr) = &stmt.expr {
            self.resolve_expr(self.ast_arena.expr_ref(*expr));
        }
    }

    fn visit_while(self, stmt: AstRef<StmtWhile>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(stmt.condition));
        self.resolve_stmt(self.ast_arena.stmt_ref(stmt.body))
    }

    fn visit_function(self, stmt: AstRef<StmtFunction>) -> Self::T {
        self.define(stmt.name.as_str().into());
        self.resolve_fn(stmt);
    }
}

impl ExprVisitor for &mut Resolver<'_, '_> {
    type T = ();

    fn visit_binary(self, expr: AstRef<ExprBinary>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(expr.left));
        self.resolve_expr(self.ast_arena.expr_ref(expr.right))
    }

    fn visit_call(self, expr: AstRef<ExprCall>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(expr.callee));
        for arg in &expr.args {
            self.resolve_expr(self.ast_arena.expr_ref(*arg));
        }
    }

    fn visit_grouping(self, expr: AstRef<ExprGrouping>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(expr.0))
    }

    fn visit_literal(self, _expr: AstRef<ExprLiteral>) -> Self::T {}

    fn visit_unary(self, expr: AstRef<ExprUnary>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(expr.right))
    }

    fn visit_variable(self, expr: AstRef<ExprVariable>) -> Self::T {
        self.resolve_local(expr.id(), &expr.name.as_str())
    }

    fn visit_assign(self, expr: AstRef<ExprAssign>) -> Self::T {
        let value = self.ast_arena.expr_ref(expr.value);
        self.resolve_expr(value);
        self.resolve_local(expr.id(), &expr.name.as_str())
    }

    fn visit_logical(self, expr: AstRef<ExprLogical>) -> Self::T {
        self.resolve_expr(self.ast_arena.expr_ref(expr.left));
        self.resolve_expr(self.ast_arena.expr_ref(expr.right))
    }
}
