use super::{environment::*, object::*};

use crate::{
    parsing::{
        expr::*,
        stmt::*,
        visitor::{ExprVisitor, StmtVisitor},
    },
    runtime::error::RuntimeError,
    tokens::TokenType,
};

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        self.env.push_scope();
        for stmt in statements {
            self.execute(stmt).inspect_err(|_| self.env.pop_scope())?;
        }
        self.env.pop_scope();
        Ok(())
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

    fn visit_grouping(&mut self, expr: &ExprGrouping) -> Self::T {
        self.evaluate(&expr.0)
    }

    fn visit_literal(&mut self, expr: &ExprLiteral) -> Self::T {
        Ok(expr.literal.clone())
    }

    fn visit_unary(&mut self, expr: &ExprUnary) -> Self::T {
        let right = self.evaluate(&expr.right)?;
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
            .ok_or(RuntimeError::undefined(&expr.name))
    }

    fn visit_assign(&mut self, expr: &ExprAssign) -> Self::T {
        let value = self.evaluate(&expr.value)?;
        self.env
            .assign(expr.name.ty.ident(), value)
            .map_err(|e| RuntimeError::custom(&expr.name, e))
    }

    fn visit_logical(&mut self, expr: &ExprLogical) -> Self::T {
        let left = self.evaluate(&expr.left)?;
        match (&expr.op.ty, left.is_truthy()) {
            (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
            (TokenType::Or, false) | (TokenType::And, true) => self.evaluate(&expr.right),
            (invalid_token, _) => panic!(
                "parsing gone wrong, token of a logical expression cannot be '{}'",
                invalid_token
            ),
        }
    }
}

impl StmtVisitor for Interpreter {
    type T = Result<(), RuntimeError>;

    fn visit_print(&mut self, stmt: &StmtPrint) -> Self::T {
        let value = self.evaluate(&stmt.expr)?;
        print!("{value}");
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
        self.execute_block(&stmt.statements)
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
}

impl Interpreter {
    pub fn interpret(&mut self, program: Vec<Stmt>) -> Result<(), RuntimeError> {
        for statement in program {
            self.execute(&statement)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
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
