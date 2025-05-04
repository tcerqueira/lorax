use std::fmt::Display;

use thiserror::Error;

use crate::{lexing::tokens::Token, span::Span};

#[derive(Debug, Error)]
#[error("[line {line}] Error <{span}>: {message}")]
pub struct ParsingError {
    pub line: u32,
    pub span: Box<str>,
    pub message: Box<str>,
}

impl ParsingError {
    pub fn custom(src: &str, token: &Token, message: impl Display) -> Self {
        Self {
            line: token.span.line_start,
            span: make_span(src, &token.span),
            message: format!("{message}").into(),
        }
    }

    pub fn expected(src: &str, expected: impl Display, found: &Token) -> Self {
        Self {
            line: found.span.line_start,
            span: make_span(src, &found.span),
            message: format!("Expected '{}', found '{}'", expected, found.ty).into(),
        }
    }
}

fn make_span(src: &str, span: &Span) -> Box<str> {
    src[span.start..span.end].into()
}
