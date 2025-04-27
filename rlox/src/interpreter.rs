use crate::{
    error::RuntimeError,
    parser::{
        expr::{Expr, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary},
        object::Object,
        visitor::Visitor,
    },
    tokens::TokenType,
};

pub struct Interpreter;

impl Visitor for Interpreter {
    type T = Result<Object, RuntimeError>;

    fn visit_binary(&mut self, expr: &ExprBinary) -> Self::T {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        let value = match expr.op.ty {
            ref op @ (TokenType::Minus | TokenType::Slash | TokenType::Star) => {
                let err_handler =
                    |e| RuntimeError::custom(&expr.op, format!("Invalid operand: {e}"));
                let left = left.try_downcast::<f64>().map_err(err_handler)?;
                let right = right.try_downcast::<f64>().map_err(err_handler)?;
                Object::new(match op {
                    TokenType::Minus => left - right,
                    TokenType::Slash => left / right,
                    TokenType::Star => left * right,
                    _ => unreachable!(),
                })
            }
            TokenType::Plus => {
                if let (Ok(left), Ok(right)) =
                    (left.try_downcast::<f64>(), right.try_downcast::<f64>())
                {
                    Object::new(left + right)
                } else if let (Ok(left), Ok(right)) = (
                    left.try_downcast::<String>(),
                    right.try_downcast::<String>(),
                ) {
                    Object::new(format!("{left}{right}"))
                } else {
                    return Err(RuntimeError::custom(
                        &expr.op,
                        "Invalid operands: Objects not both String or f64",
                    ));
                }
            }
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
}

impl Interpreter {
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
        Parser::parse(tokens)
            .inspect_err(|e| eprintln!("{e}"))
            .expect("syntax error")
    }

    #[test]
    fn interpret_unary_bang() -> anyhow::Result<()> {
        let ast = expr("!9");
        let value = Interpreter.interpret(&ast)?;
        assert!(!*value.downcast::<bool>());

        let ast = expr("!\"hello\"");
        let value = Interpreter.interpret(&ast)?;
        assert!(!*value.downcast::<bool>());

        let ast = expr("!-0");
        let value = Interpreter.interpret(&ast)?;
        assert!(!*value.downcast::<bool>());

        let ast = expr("!false");
        let value = Interpreter.interpret(&ast)?;
        assert!(*value.downcast::<bool>());

        let ast = expr("!(1 - 1)");
        let value = Interpreter.interpret(&ast)?;
        assert!(!*value.downcast::<bool>());
        Ok(())
    }

    #[test]
    fn interpret_unary_minus() -> anyhow::Result<()> {
        let ast = expr("-1");
        let value = Interpreter.interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), -1.);

        let ast = expr("--1");
        let value = Interpreter.interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 1.);

        let ast = expr("-(-1 - 2)");
        let value = Interpreter.interpret(&ast)?;
        assert_eq!(*value.downcast::<f64>(), 3.);
        Ok(())
    }
}
