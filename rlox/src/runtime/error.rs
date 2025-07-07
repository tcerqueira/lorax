use std::fmt::Display;

use thiserror::Error;

use crate::{
    lexing::tokens::Token,
    report::{Report, span::Span},
    runtime::callable::CallError,
};

#[derive(Debug, Error)]
#[error("[line {}:{}] {message}", .span.line_start, .span.start)]
pub struct RuntimeError {
    pub span: Span,
    pub message: Box<str>,
}

impl RuntimeError {
    pub fn custom(token: &Token, message: impl Display) -> Self {
        Self {
            span: token.span.clone(),
            message: format!("{message}").into(),
        }
    }

    pub fn undefined(found: &Token) -> Self {
        Self {
            span: found.span.clone(),
            message: "Undefined variable.".into(),
        }
    }
}

impl Report for RuntimeError {
    fn report(&self, _source: &str) {
        eprint!("{}", self.message);
    }

    fn span(&self) -> &Span {
        &self.span
    }
}

impl From<CallError> for RuntimeError {
    fn from(err: CallError) -> Self {
        Self {
            span: err.span().clone(),
            message: err.to_string().into_boxed_str(),
        }
    }
}
