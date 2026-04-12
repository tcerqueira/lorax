use std::fmt::Display;

use thiserror::Error;

use crate::{Report, Spanned, span::Span};

#[derive(Debug, Error)]
#[error("[line {}:{}] {message}", .span.line_start, .span.start)]
pub struct Error {
    pub span: Span,
    pub message: Box<str>,
}

impl Error {
    pub fn custom(spanned: impl Spanned, message: impl Display) -> Self {
        Self {
            span: spanned.span(),
            message: format!("{message}").into(),
        }
    }

    pub fn with_token(spanned: impl Spanned, message: impl Display) -> Self {
        Self {
            span: spanned.span(),
            message: format!("{message}").into(),
        }
    }

    pub fn undefined(spanned: impl Spanned) -> Self {
        Self {
            span: spanned.span(),
            message: "Undefined variable.".into(),
        }
    }

    pub fn not_callable(span: Span) -> Self {
        Self {
            span,
            message: "Object is not a callable.".into(),
        }
    }

    pub fn arity(span: Span, expected: u8, found: usize) -> Self {
        Self {
            span,
            message: format!("Expected {expected} arguments but found {found}").into(),
        }
    }

    pub fn invalid_break_or_continue(spanned: impl Spanned) -> Self {
        Self {
            span: spanned.span(),
            message: "Invalid control flow statement outside for/while loop.".into(),
        }
    }

    pub fn invalid_return(spanned: impl Spanned) -> Self {
        Self {
            span: spanned.span(),
            message: "Invalid return statement function.".into(),
        }
    }
}

impl Report for Error {
    fn report(&self, _source: &str) {
        eprint!("{}", self.message);
    }
}

impl Spanned for Error {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
