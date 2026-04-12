use std::fmt::Display;

use thiserror::Error;

use crate::{Report, Span, Spanned};

#[derive(Debug, Error)]
#[error("[line {}:{}] {message}", .span.line_start, .span.start)]
pub struct Error {
    pub span: Span,
    pub message: Box<str>,
    pub should_sync: bool,
}

impl Error {
    pub fn custom(spanned: impl Spanned, message: impl Display) -> Self {
        Self {
            span: spanned.span(),
            message: format!("{message}").into(),
            should_sync: true,
        }
    }

    pub fn custom_no_sync(spanned: impl Spanned, message: impl Display) -> Self {
        Self {
            span: spanned.span(),
            message: format!("{message}").into(),
            should_sync: false,
        }
    }

    pub fn expected(spanned: impl Spanned, expected: impl Display, found: impl Display) -> Self {
        Self {
            span: spanned.span(),
            message: format!("Expected '{}', found '{}'", expected, found).into(),
            should_sync: true,
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
