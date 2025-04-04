mod expr;
mod visitor;

use crate::tokens::*;
use expr::*;

// expression   => equality;
// equality     => comparison ( ("!=" | "==") comparison )*;
// comparison   => term ( (">" | ">=" | "<" | "<=") term )*;
// term         => factor ( ("-" | "+") factor )*;
// factor       => unary ( ("/" | "*") unary )*;
// unary        => ("!" | "-") unary
//              | primary;
// primary      => NUMBER
//              | STRING
//              | "true"
//              | "false"
//              | "nil"
//              | "(" expression ")";

pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
}

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Expr {
        let mut parser = Self { tokens, curr: 0 };
        parser.expression()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        todo!()
    }

    fn comparison(&mut self) -> Expr {
        todo!()
    }

    fn term(&mut self) -> Expr {
        todo!()
    }

    fn factor(&mut self) -> Expr {
        todo!()
    }

    fn unary(&mut self) -> Expr {
        todo!()
    }

    fn primary(&mut self) -> Expr {
        todo!()
    }
}
