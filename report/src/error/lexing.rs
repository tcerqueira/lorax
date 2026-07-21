use thiserror::Error;

use crate::{Report, Span, Spanned};

#[derive(Debug, Error, Clone)]
#[error("[line {}:{}] {}", (.span).line_start, .span.start, .message)]
pub struct Error {
    pub span: Span,
    pub message: Box<str>,
}

impl Error {
    pub fn new(spanned: impl Spanned, message: Box<str>) -> Self {
        Self {
            span: spanned.span(),
            message,
        }
    }
}

impl Report for Error {
    fn report(&self, _source: &str, w: &mut dyn std::io::Write) {
        let _ = write!(w, "{}", self.message);
    }
}

impl Spanned for Error {
    fn span(&self) -> Span {
        self.span
    }
}
