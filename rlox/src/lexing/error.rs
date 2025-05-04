use std::fmt::Display;

use thiserror::Error;

use crate::report::Span;

#[derive(Debug, Error)]
#[error("[line {line}] Error <{span}>: {message}")]
pub struct LexingError {
    pub line: u32,
    pub span: Box<str>,
    pub message: Box<str>,
}

impl LexingError {
    pub fn custom(src: &str, span: &Span, message: impl Display) -> Self {
        Self {
            line: span.line_start,
            span: make_span(src, span),
            message: format!("{message}").into(),
        }
    }
}

fn make_span(src: &str, span: &Span) -> Box<str> {
    src[span.start..span.end].into()
}
