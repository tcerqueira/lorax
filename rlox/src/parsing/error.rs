use std::fmt::Display;

use thiserror::Error;

use crate::{
    lexing::tokens::Token,
    report::{Report, Span},
};

#[derive(Debug, Error)]
#[error("[line {}:{}] {message}", .span.line_start, .span.start)]
pub struct ParsingError {
    pub span: Span,
    pub message: Box<str>,
}

impl ParsingError {
    pub fn custom(token: &Token, message: impl Display) -> Self {
        Self {
            span: token.span.clone(),
            message: format!("{message}").into(),
        }
    }

    pub fn expected(expected: impl Display, found: &Token) -> Self {
        Self {
            span: found.span.clone(),
            message: format!("Expected '{}', found '{}'", expected, found.ty).into(),
        }
    }
}

impl Report for ParsingError {
    fn report(&self, _source: &str) {
        eprint!("{}", self.message);
    }

    fn span(&self) -> &Span {
        &self.span
    }
}
