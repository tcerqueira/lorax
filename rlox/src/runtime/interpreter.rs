use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use super::{environment::*, object::*};

use crate::{
    parsing::{
        expr::*,
        stmt::*,
        visitor::{ExprVisitor, StmtVisitor},
    },
    report::{Span, Spanned},
    runtime::{
        callable::{Function, NativeFunction},
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

    pub fn interpret(&mut self, program: Vec<Stmt>) -> Result<(), RuntimeError> {
        for statement in program {
            self.execute(&statement)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        let mut this = self.new_span(expr.span());
        expr.accept(&mut *this)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    pub(super) fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
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

impl ExprVisitor for Interpreter {
    type T = Result<Object, RuntimeError>;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let err_handler = |e| RuntimeError::custom(&expr.op, e);

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

    fn visit_call(&mut self, expr: &ExprCall) -> Self::T {
        let mut this = self.new_span(expr.callee.span());
        let callee = this.evaluate(&expr.callee)?;
        let args = expr
            .args
            .iter()
            .map(|arg| this.evaluate(arg))
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

        let result = callable.call(&mut this, args)?;
        Ok(result)
    }

    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T {
        self.evaluate(&expr.0)
    }

    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T {
        Ok(expr.literal.clone())
    }

    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T {
        let mut this = self.new_span(expr.span());
        let right = this.evaluate(&expr.right)?;
        let value =
            match expr.op.ty {
                TokenType::Minus => Object::new(-right.try_downcast::<f64>().map_err(|e| {
                    RuntimeError::custom(&expr.op, format!("Invalid operand: {e}"))
                })?),
                TokenType::Bang => Object::new(!right.is_truthy()),
                _ => panic!("Unexpected unary operator: {:?}", expr.op),
            };

        Ok(value)
    }

    fn visit_variable(&mut self, expr: &ExprVariable) -> Self::T {
        self.env
            .get(expr.name.ty.ident())
            .ok_or_else(|| RuntimeError::undefined(&expr.name))
    }

    fn visit_assign(&mut self, expr: &ExprAssign) -> Self::T {
        let mut this = self.new_span(expr.span());
        let value = this.evaluate(&expr.value)?;
        this.env
            .assign(expr.name.ty.ident(), value)
            .map_err(|e| RuntimeError::custom(&expr.name, e))
    }

    fn visit_logical(&mut self, expr: &ExprLogical) -> Self::T {
        let mut this = self.new_span(expr.span());
        let left = this.evaluate(&expr.left)?;
        match (&expr.op.ty, left.is_truthy()) {
            (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
            (TokenType::Or, false) | (TokenType::And, true) => this.evaluate(&expr.right),
            (invalid_token, _) => unreachable!(
                "parsing gone wrong, token of a logical expression cannot be '{invalid_token}'"
            ),
        }
    }
}

impl StmtVisitor for Interpreter {
    type T = Result<(), RuntimeError>;

    fn visit_print(&mut self, stmt: &StmtPrint) -> Self::T {
        let value = self.evaluate(&stmt.expr)?;
        println!("{value}");
        Ok(())
    }

    fn visit_expression(&mut self, stmt: &StmtExpression) -> Self::T {
        self.evaluate(&stmt.expr)?;
        Ok(())
    }

    fn visit_var(&mut self, stmt: &StmtVar) -> Self::T {
        let initializer = stmt
            .initializer
            .as_ref()
            .map(|e| self.evaluate(e))
            .transpose()?
            .unwrap_or_else(Object::nil);

        self.env.define(stmt.ident.ty.ident().into(), initializer);
        Ok(())
    }

    fn visit_block(&mut self, stmt: &StmtBlock) -> Self::T {
        let mut scope = self.new_env();
        scope.execute_block(&stmt.statements)
    }

    fn visit_if(&mut self, stmt: &StmtIf) -> Self::T {
        if self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.then_branch)
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_while(&mut self, stmt: &StmtWhile) -> Self::T {
        while self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.body)?;
        }
        Ok(())
    }

    fn visit_function(&mut self, stmt: &StmtFunction) -> Self::T {
        let stmt = stmt.clone();
        self.env.define(
            stmt.name.ty().ident().into(),
            Object::new(Function::new(stmt)),
        );
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
    use crate::{lexing::Scanner, parsing::Parser};

    use super::*;

    fn expr(source: &str) -> Expr {
        let tokens = Scanner::new(source)
            .scan_tokens()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("token error");
        Parser::new(tokens)
            .expression()
            .inspect_err(|e| eprintln!("{e}"))
            .expect("syntax error")
    }

    #[test]
    fn interpret_unary_bang() -> anyhow::Result<()> {
        let src = "!9";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!\"hello\"";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!-0";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!false";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert!(*value.downcast::<bool>());

        let src = "!(1 - 1)";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());
        Ok(())
    }

    #[test]
    fn interpret_unary_minus() -> anyhow::Result<()> {
        let src = "-1";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);

        let src = "--1";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 1.);

        let src = "-(-1 - 2)";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);
        Ok(())
    }

    #[test]
    fn interpret_unary_minus_err() -> anyhow::Result<()> {
        let src = "-\"h\"";
        let ast = expr(src);
        Interpreter::new()
            .evaluate(&ast)
            .expect_err("can't negate strings");
        Ok(())
    }

    #[test]
    fn interpret_binary_plus() -> anyhow::Result<()> {
        let src = "1 + 2";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);

        let src = "\"Hello\" + \" \" + \"World!\"";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert_eq!(*value.downcast::<String>(), "Hello World!");

        let src = "1 + -2";
        let ast = expr(src);
        let value = Interpreter::new().evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);
        Ok(())
    }

    #[test]
    fn interpret_binary_plus_err() -> anyhow::Result<()> {
        let src = "\"h\" + 1";
        let ast = expr(src);
        Interpreter::new()
            .evaluate(&ast)
            .expect_err("can't add strings and numbers");
        Ok(())
    }

    fn program(source: &str) -> Vec<Stmt> {
        let tokens = Scanner::new(source)
            .scan_tokens()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("token error");
        Parser::new(tokens)
            .parse()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("syntax error")
    }

    #[test]
    fn test_examples() {
        let lox_examples = std::fs::read_dir("./examples")
            .unwrap()
            .flatten()
            .filter(|f| f.file_name().into_string().unwrap().ends_with(".lox"))
            .map(|f| (f.path(), std::fs::read_to_string(f.path())));

        for (path, src) in lox_examples {
            let src = src.unwrap_or_else(|e| panic!("could not open example file {path:?}: {e:?}"));
            let ast = program(&src);
            Interpreter::new()
                .interpret(ast)
                .expect("program runs successfully");
        }
    }
}
