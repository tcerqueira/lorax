use std::fmt::Display;

use thiserror::Error;

use crate::{lexing::tokens::Token, parsing::expr::Expr, span::Span};

#[derive(Debug, Error)]
#[error("[line {line}] Error <{span}>: {message}")]
pub struct RuntimeError {
    pub line: u32,
    pub span: Box<str>,
    pub message: Box<str>,
}

impl RuntimeError {
    pub fn custom(src: &str, expr: &Expr, message: impl Display) -> Self {
        let span = expr.span();
        Self {
            line: span.line_start,
            span: make_span(src, &span),
            message: format!("{message}").into(),
        }
    }

    pub fn undefined(src: &str, token: &Token) -> Self {
        Self {
            line: token.span.line_start,
            span: make_span(src, &token.span),
            message: "Undefined variable.".into(),
        }
    }
}

fn make_span(src: &str, span: &Span) -> Box<str> {
    src[span.start..span.end].into()
}
