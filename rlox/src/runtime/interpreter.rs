use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use super::{environment::*, object::*};

use crate::{
    parsing::{
        ast::{AstArena, AstRef, ExprId, ExprRef, StmtId, StmtRef},
        expr::*,
        stmt::*,
        visitor::{ExprVisitor, StmtVisitor},
    },
    report::{Span, Spanned},
    runtime::{
        callable::{Function, NativeFunction},
        control_flow::ControlFlow,
        error::RuntimeError,
    },
    tokens::TokenType,
};

pub struct Interpreter {
    pub(super) env: Environment,
    pub(super) span_stack: VecDeque<Span>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut this = Self {
            env: Environment::new(),
            span_stack: vec![Span::default()].into(),
        };
        this.define_builtins();

        this
    }

    pub fn interpret(
        &mut self,
        program: Vec<StmtId>,
        ast_arena: &AstArena,
    ) -> Result<(), RuntimeError> {
        for statement in program.iter().map(|&s| ast_arena.stmt_ref(s)) {
            match self.execute(statement) {
                Ok(()) => {}
                Err(cf @ (ControlFlow::Break | ControlFlow::Continue)) => {
                    return Err(RuntimeError::invalid_break_or_continue(
                        self.current_span(),
                        cf,
                    ));
                }
                Err(ControlFlow::Return(_)) => {
                    return Err(RuntimeError::invalid_return(self.current_span()));
                }
                Err(ControlFlow::Error(runtime)) => return Err(runtime),
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: ExprRef) -> Result<Object, RuntimeError> {
        let mut this = self.new_span(expr.span());
        expr.accept(&mut *this)
    }

    fn execute(&mut self, stmt: StmtRef) -> Result<(), ControlFlow> {
        stmt.accept(self)
    }

    pub(crate) fn resolve(&mut self, expr_id: ExprId, depth: usize) {
        self.env.resolve_var(expr_id, depth);
    }

    pub(super) fn execute_block<'a>(
        &mut self,
        statements: impl IntoIterator<Item = StmtRef<'a>>,
    ) -> Result<(), ControlFlow> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    pub(super) fn new_env(&mut self) -> InterpreterScope<'_, impl FnMut(&mut Interpreter)> {
        self.env.push_scope();
        InterpreterScope::new(self, |i| i.env.pop_scope())
    }

    pub(super) fn new_span(
        &mut self,
        span: Span,
    ) -> InterpreterScope<'_, impl FnMut(&mut Interpreter)> {
        self.span_stack.push_front(span);
        InterpreterScope::new(self, |i| {
            i.span_stack.pop_front();
        })
    }

    pub(super) fn current_span(&self) -> &Span {
        self.span_stack.front().expect("there's always root span")
    }

    fn define_builtins(&mut self) {
        self.env.define_global(
            "clock".into(),
            Object::new(NativeFunction::new("clock", 0, |_interpreter, _args| {
                Ok(Object::new(
                    std::time::UNIX_EPOCH
                        .elapsed()
                        .expect("couldn't get system time")
                        .as_millis(),
                ))
            })),
        );
    }
}

impl ExprVisitor for &mut Interpreter {
    type T = Result<Object, RuntimeError>;

    fn visit_binary(self, expr: AstRef<ExprBinary>) -> Self::T {
        let arena = expr.arena();
        let left = self.evaluate(arena.expr_ref(expr.left))?;
        let right = self.evaluate(arena.expr_ref(expr.right))?;
        let err_handler = |e| RuntimeError::with_token(&expr.op, e);

        let value = match expr.op.ty {
            TokenType::Plus => (left + right).map_err(err_handler)?,
            TokenType::Minus => (left - right).map_err(err_handler)?,
            TokenType::Star => (left * right).map_err(err_handler)?,
            TokenType::Slash => (left / right).map_err(err_handler)?,
            // FIXME: call partial_cmp and handle None case ?
            TokenType::Greater => Object::new(left > right),
            TokenType::GreaterEqual => Object::new(left >= right),
            TokenType::Less => Object::new(left < right),
            TokenType::LessEqual => Object::new(left <= right),
            TokenType::EqualEqual => Object::new(left == right),
            TokenType::BangEqual => Object::new(left != right),
            _ => panic!("Unexpected binary operator: {:?}", expr.op),
        };

        Ok(value)
    }

    fn visit_call(self, expr: AstRef<ExprCall>) -> Self::T {
        let arena = expr.arena();

        let callee = arena.expr_ref(expr.callee);
        let mut this = self.new_span(callee.span());
        let callee = this.evaluate(callee)?;
        let args = expr
            .args
            .iter()
            .map(|arg| this.evaluate(arena.expr_ref(*arg)))
            .collect::<Result<Vec<_>, _>>()?;

        let callable = callee
            .as_callable()
            .ok_or_else(|| RuntimeError::not_callable(this.current_span().clone()))?;

        if callable.arity() as usize != args.len() {
            return Err(RuntimeError::arity(
                this.current_span().clone(),
                callable.arity(),
                args.len(),
            ));
        }

        let result = callable.call(&mut this, arena, args)?;
        Ok(result)
    }

    fn visit_grouping(self, expr: AstRef<ExprGrouping>) -> Self::T {
        self.evaluate(expr.arena().expr_ref(expr.0))
    }

    fn visit_literal(self, expr: AstRef<ExprLiteral>) -> Self::T {
        Ok(expr.literal.clone())
    }

    fn visit_unary(self, expr: AstRef<ExprUnary>) -> Self::T {
        let arena = expr.arena();
        let mut this = self.new_span(expr.span());
        let right = this.evaluate(arena.expr_ref(expr.right))?;
        let value = match expr.op.ty {
            TokenType::Minus => Object::new(-right.try_downcast::<f64>().map_err(|e| {
                RuntimeError::with_token(&expr.op, format!("Invalid operand: {e}"))
            })?),
            TokenType::Bang => Object::new(!right.is_truthy()),
            _ => panic!("Unexpected unary operator: {:?}", expr.op),
        };

        Ok(value)
    }

    fn visit_variable(self, expr: AstRef<ExprVariable>) -> Self::T {
        self.env
            .get(expr)
            .ok_or_else(|| RuntimeError::undefined(&expr.name))
    }

    fn visit_assign(self, expr: AstRef<ExprAssign>) -> Self::T {
        let arena = expr.arena();
        let mut this = self.new_span(expr.span());
        let value = this.evaluate(arena.expr_ref(expr.value))?;
        this.env
            .assign(expr, value)
            .map_err(|e| RuntimeError::with_token(&expr.name, e))
    }

    fn visit_logical(self, expr: AstRef<ExprLogical>) -> Self::T {
        let arena = expr.arena();
        let mut this = self.new_span(expr.span());
        let left = this.evaluate(arena.expr_ref(expr.left))?;
        match (&expr.op.ty, left.is_truthy()) {
            (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
            (TokenType::Or, false) | (TokenType::And, true) => {
                this.evaluate(arena.expr_ref(expr.right))
            }
            (invalid_token, _) => unreachable!(
                "parsing gone wrong, token of a logical expression cannot be '{invalid_token}'"
            ),
        }
    }
}

impl StmtVisitor for &mut Interpreter {
    type T = Result<(), ControlFlow>;

    fn visit_print(self, stmt: AstRef<StmtPrint>) -> Self::T {
        let arena = stmt.arena();
        let value = self.evaluate(arena.expr_ref(stmt.expr))?;
        println!("{value}");
        Ok(())
    }

    fn visit_expression(self, stmt: AstRef<StmtExpression>) -> Self::T {
        let arena = stmt.arena();
        self.evaluate(arena.expr_ref(stmt.expr))?;
        Ok(())
    }

    fn visit_var(self, stmt: AstRef<StmtVar>) -> Self::T {
        let arena = stmt.arena();
        let initializer = stmt
            .initializer
            .map(|init| arena.expr_ref(init))
            .map(|e| self.evaluate(e))
            .transpose()?
            .unwrap_or_else(Object::nil);

        self.env.define(stmt.ident.as_str().into(), initializer);
        Ok(())
    }

    fn visit_block(self, stmt: AstRef<StmtBlock>) -> Self::T {
        let arena = stmt.arena();
        let mut scope = self.new_env();
        scope.execute_block(stmt.statements.iter().map(|&s| arena.stmt_ref(s)))
    }

    fn visit_if(self, stmt: AstRef<StmtIf>) -> Self::T {
        let arena = stmt.arena();

        if self.evaluate(arena.expr_ref(stmt.condition))?.is_truthy() {
            self.execute(arena.stmt_ref(stmt.then_branch))
        } else if let Some(else_branch) = stmt.else_branch {
            self.execute(arena.stmt_ref(else_branch))
        } else {
            Ok(())
        }
    }

    fn visit_return(self, stmt: AstRef<StmtReturn>) -> Self::T {
        let arena = stmt.arena();
        Err(ControlFlow::Return(
            match stmt.expr.map(|e| arena.expr_ref(e)) {
                Some(expr) => self.evaluate(expr)?,
                None => Object::nil(),
            },
        ))
    }

    fn visit_while(self, stmt: AstRef<StmtWhile>) -> Self::T {
        let arena = stmt.arena();

        while self.evaluate(arena.expr_ref(stmt.condition))?.is_truthy() {
            self.execute(arena.stmt_ref(stmt.body))?;
        }
        Ok(())
    }

    fn visit_function(self, stmt: AstRef<StmtFunction>) -> Self::T {
        self.env
            .define(stmt.name.as_str().into(), Object::new(Function::new(stmt)));
        Ok(())
    }
}

pub struct InterpreterScope<'i, F>
where
    F: FnMut(&mut Interpreter),
{
    interpreter: &'i mut Interpreter,
    drop_fn: F,
}

impl<'i, F> InterpreterScope<'i, F>
where
    F: FnMut(&mut Interpreter),
{
    pub fn new(interpreter: &'i mut Interpreter, drop_fn: F) -> Self {
        Self {
            interpreter,
            drop_fn,
        }
    }
}

impl<'i, F> Drop for InterpreterScope<'i, F>
where
    F: FnMut(&mut Interpreter),
{
    fn drop(&mut self) {
        (self.drop_fn)(self.interpreter)
    }
}

impl<'i, F> Deref for InterpreterScope<'i, F>
where
    F: FnMut(&mut Interpreter),
{
    type Target = Interpreter;

    fn deref(&self) -> &Self::Target {
        self.interpreter
    }
}

impl<'i, F> DerefMut for InterpreterScope<'i, F>
where
    F: FnMut(&mut Interpreter),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.interpreter
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexing::Scanner, parsing::Parser, passes::resolver::Resolver};

    use super::*;

    fn expr(source: &str, ast_arena: &mut AstArena) -> Expr {
        let tokens = Scanner::new(source)
            .scan_tokens()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("token error");
        Parser::new(ast_arena, tokens)
            .expression()
            .inspect_err(|e| eprintln!("{e}"))
            .expect("syntax error")
    }

    #[test]
    fn interpret_unary_bang() -> anyhow::Result<()> {
        let mut ast_arena = AstArena::default();

        let src = "!9";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert!(!*value.downcast::<bool>());

        let src = "!\"hello\"";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert!(!*value.downcast::<bool>());

        let src = "!-0";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert!(!*value.downcast::<bool>());

        let src = "!false";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert!(*value.downcast::<bool>());

        let src = "!(1 - 1)";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert!(!*value.downcast::<bool>());

        Ok(())
    }

    #[test]
    fn interpret_unary_minus() -> anyhow::Result<()> {
        let mut ast_arena = AstArena::default();

        let src = "-1";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert_eq!(*value.downcast::<f64>(), -1.);

        let src = "--1";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert_eq!(*value.downcast::<f64>(), 1.);

        let src = "-(-1 - 2)";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert_eq!(*value.downcast::<f64>(), 3.);
        Ok(())
    }

    #[test]
    fn interpret_unary_minus_err() -> anyhow::Result<()> {
        let mut ast_arena = AstArena::default();

        let src = "-\"h\"";
        let ast = expr(src, &mut ast_arena);
        Interpreter::new()
            .evaluate(ast_arena.alloc_expr(ast))
            .expect_err("can't negate strings");
        Ok(())
    }

    #[test]
    fn interpret_binary_plus() -> anyhow::Result<()> {
        let mut ast_arena = AstArena::default();

        let src = "1 + 2";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert_eq!(*value.downcast::<f64>(), 3.);

        let src = "\"Hello\" + \" \" + \"World!\"";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert_eq!(*value.downcast::<String>(), "Hello World!");

        let src = "1 + -2";
        let ast = expr(src, &mut ast_arena);
        let value = Interpreter::new().evaluate(ast_arena.alloc_expr(ast))?;
        assert_eq!(*value.downcast::<f64>(), -1.);
        Ok(())
    }

    #[test]
    fn interpret_binary_plus_err() -> anyhow::Result<()> {
        let mut ast_arena = AstArena::default();

        let src = "\"h\" + 1";
        let ast = expr(src, &mut ast_arena);
        Interpreter::new()
            .evaluate(ast_arena.alloc_expr(ast))
            .expect_err("can't add strings and numbers");
        Ok(())
    }

    fn program(source: &str, ast_arena: &mut AstArena) -> Vec<StmtId> {
        let tokens = Scanner::new(source)
            .scan_tokens()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("token error");
        Parser::new(ast_arena, tokens)
            .parse()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("syntax error")
    }

    #[test]
    fn test_examples() {
        let mut ast_arena = AstArena::default();

        let lox_examples = std::fs::read_dir("./examples")
            .unwrap()
            .flatten()
            .filter(|f| f.file_name().into_string().unwrap().ends_with(".lox"))
            .map(|f| (f.path(), std::fs::read_to_string(f.path())));

        for (path, src) in lox_examples {
            let src = src.unwrap_or_else(|e| panic!("could not open example file {path:?}: {e:?}"));
            let ast = program(&src, &mut ast_arena);

            let mut interpreter = Interpreter::new();
            Resolver::new(&mut interpreter, &ast_arena)
                .resolve_stmts(&ast)
                .expect("resolver failed to resolve");

            interpreter
                .interpret(ast, &ast_arena)
                .expect("program runs successfully");
        }
    }
}
