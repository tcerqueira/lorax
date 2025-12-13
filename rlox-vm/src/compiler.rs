use std::iter::Peekable;

use anyhow::Context;
use rlox_lexer::{Scanner, tokens::Token};
use rlox_report::error::LexingError;

use crate::chunk::Chunk;

pub struct Compiler<'s> {
    scanner: Peekable<Scanner<'s>>,
}

impl<'s> Compiler<'s> {
    pub fn new(scanner: Scanner<'s>) -> Self {
        Self {
            scanner: scanner.peekable(),
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, anyhow::Error> {
        while let Some(token) = self.advance().context("hehe")? {
            println!("{token}");
        }
        self.expression();
        todo!()
    }

    fn expression(&mut self) {
        todo!()
    }

    fn advance(&mut self) -> Result<Option<Token>, LexingError> {
        self.scanner.next().transpose()
    }

    #[expect(dead_code)]
    fn peek(&mut self) -> Option<&Result<Token, LexingError>> {
        self.scanner.peek()
    }
}

// #[derive(Debug, Error)]
// pub struct CompilerError {
//     #[from]
//     source: anyhow::Error,
// }

// impl Display for CompilerError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "{}", self.source)?;
//         if let Some(cause) = self.source.source() {
//             for cause in cause.sources() {
//                 writeln!(f, "{:>10}{cause}", '|')?;
//             }
//         }
//         Ok(())
//     }
// }
