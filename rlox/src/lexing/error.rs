use thiserror::Error;

use crate::report::{Report, Span, Spanned};

#[derive(Debug, Error)]
#[error("[line {}:{}] {}", (.span).line_start, .span.start, .message)]
pub struct LexingError {
    pub span: Span,
    pub message: Box<str>,
}

impl LexingError {
    pub fn new(span: Span, message: Box<str>) -> Self {
        Self { span, message }
    }
}

impl Report for LexingError {
    fn report(&self, _source: &str) {
        eprint!("{}", self.message);
    }
}

impl Spanned for LexingError {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
