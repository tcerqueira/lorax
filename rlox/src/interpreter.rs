use crate::{
    error::RuntimeError,
    parser::{
        expr::{Expr, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary},
        object::Object,
        visitor::Visitor,
    },
    tokens::TokenType,
};

pub struct Interpreter<'s> {
    src: &'s str,
}

impl<'s> Interpreter<'s> {
    pub fn new(src: &'s str) -> Self {
        Self { src }
    }
}

impl Visitor for Interpreter<'_> {
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
}

impl Interpreter<'_> {
    pub fn interpret(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        self.evaluate(expr)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        expr.accept(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner};

    use super::*;

    fn expr(source: &str) -> Expr {
        let scanner = Scanner::new(source);
        let tokens = scanner
            .scan_tokens()
            .inspect_err(|errs| errs.iter().for_each(|e| eprintln!("{e}")))
            .expect("token error");
        Parser::parse(source, tokens)
            .inspect_err(|e| eprintln!("{e}"))
            .expect("syntax error")
    }

    #[test]
    fn interpret_unary_bang() -> anyhow::Result<()> {
        let src = "!9";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!\"hello\"";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!-0";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert!(!*value.downcast::<bool>());

        let src = "!false";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert!(*value.downcast::<bool>());

        let src = "!(1 - 1)";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert!(!*value.downcast::<bool>());
        Ok(())
    }

    #[test]
    fn interpret_unary_minus() -> anyhow::Result<()> {
        let src = "-1";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);

        let src = "--1";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 1.);

        let src = "-(-1 - 2)";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);
        Ok(())
    }

    #[test]
    fn interpret_unary_minus_err() -> anyhow::Result<()> {
        let src = "-\"h\"";
        let ast = expr(src);
        Interpreter::new(src)
            .interpret(&ast)
            .expect_err("can't negate strings");
        Ok(())
    }

    #[test]
    fn interpret_binary_plus() -> anyhow::Result<()> {
        let src = "1 + 2";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);

        let src = "\"Hello\" + \" \" + \"World!\"";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert_eq!(*value.downcast::<String>(), "Hello World!");

        let src = "1 + -2";
        let ast = expr(src);
        let value = Interpreter::new(src).interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);
        Ok(())
    }

    #[test]
    fn interpret_binary_plus_err() -> anyhow::Result<()> {
        let src = "\"h\" + 1";
        let ast = expr(src);
        Interpreter::new(src)
            .interpret(&ast)
            .expect_err("can't add strings and numbers");
        Ok(())
    }
}
