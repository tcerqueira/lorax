use environment::*;
use object::*;

use crate::{
    interpreter::error::RuntimeError,
    parser::{
        expr::*,
        stmt::*,
        visitor::{ExprVisitor, StmtVisitor},
    },
    tokens::TokenType,
};

pub mod environment;
pub mod error;
pub mod object;

pub struct Interpreter<'s> {
    src: &'s str,
    env: Environment,
}

impl<'s> Interpreter<'s> {
    pub fn new(src: &'s str) -> Self {
        Self {
            src,
            env: Environment::new(),
        }
    }
}

impl ExprVisitor for Interpreter<'_> {
    type T = Result<Object, RuntimeError>;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let err_handler = |e| RuntimeError::custom(self.src, &expr.clone().into(), e);

        let value = match expr.op.ty {
            TokenType::Plus => (left + right).map_err(err_handler)?,
            TokenType::Minus => (left - right).map_err(err_handler)?,
            TokenType::Star => (left * right).map_err(err_handler)?,
            TokenType::Slash => (left / right).map_err(err_handler)?,
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
        let value = match expr.op.ty {
            TokenType::Minus => Object::new(-right.try_downcast::<f64>().map_err(|e| {
                RuntimeError::custom(self.src, &expr.right, format!("Invalid operand: {e}"))
            })?),
            TokenType::Bang => Object::new(!right.is_truthy()),
            _ => panic!("Unexpected unary operator: {:?}", expr.op),
        };
        Ok(value)
    }

    fn visit_variable(&mut self, expr: &ExprVariable) -> Self::T {
        self.env
            .get(expr.name.ty.ident())
            .ok_or(RuntimeError::undefined(self.src, &expr.name))
    }

    fn visit_assign(&mut self, expr: &ExprAssign) -> Self::T {
        let value = self.evaluate(&expr.value)?;
        self.env
            .assign(expr.name.ty.ident(), value)
            .map_err(|e| RuntimeError::custom(self.src, &expr.value, e))
    }
}

impl StmtVisitor for Interpreter<'_> {
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
}

impl Interpreter<'_> {
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
    use crate::{lexer::Scanner, parser::Parser};

    use super::*;

    fn expr(source: &str) -> Expr {
        let tokens = Scanner::new(source)
            .scan_tokens()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("token error");
        Parser::new(source, tokens)
            .expression()
            .inspect_err(|e| eprintln!("{e}"))
            .expect("syntax error")
    }

    #[test]
    fn interpret_unary_bang() -> anyhow::Result<()> {
        let src = "!9";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!\"hello\"";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!-0";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!false";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert!(*value.downcast::<bool>());

        let src = "!(1 - 1)";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert!(!*value.downcast::<bool>());
        Ok(())
    }

    #[test]
    fn interpret_unary_minus() -> anyhow::Result<()> {
        let src = "-1";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);

        let src = "--1";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 1.);

        let src = "-(-1 - 2)";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);
        Ok(())
    }

    #[test]
    fn interpret_unary_minus_err() -> anyhow::Result<()> {
        let src = "-\"h\"";
        let ast = expr(src);
        Interpreter::new(src)
            .evaluate(&ast)
            .expect_err("can't negate strings");
        Ok(())
    }

    #[test]
    fn interpret_binary_plus() -> anyhow::Result<()> {
        let src = "1 + 2";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);

        let src = "\"Hello\" + \" \" + \"World!\"";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert_eq!(*value.downcast::<String>(), "Hello World!");

        let src = "1 + -2";
        let ast = expr(src);
        let value = Interpreter::new(src).evaluate(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);
        Ok(())
    }

    #[test]
    fn interpret_binary_plus_err() -> anyhow::Result<()> {
        let src = "\"h\" + 1";
        let ast = expr(src);
        Interpreter::new(src)
            .evaluate(&ast)
            .expect_err("can't add strings and numbers");
        Ok(())
    }
}
