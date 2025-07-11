use std::fmt::Display;

use thiserror::Error;

use crate::{
    lexing::tokens::Token,
    report::{Report, Spanned, span::Span},
    runtime::control_flow::ControlFlow,
};

#[derive(Debug, Error)]
#[error("[line {}:{}] {message}", .span.line_start, .span.start)]
pub struct RuntimeError {
    pub span: Span,
    pub message: Box<str>,
}

impl RuntimeError {
    #[expect(dead_code)]
    pub fn custom(spanned: impl Spanned, message: impl Display) -> Self {
        Self {
            span: spanned.span(),
            message: format!("{message}").into(),
        }
    }

    pub fn with_token(token: &Token, message: impl Display) -> Self {
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

    pub fn invalid_break_or_continue(spanned: impl Spanned, cf: ControlFlow) -> Self {
        Self {
            span: spanned.span(),
            message: format!("Invalid {cf} control flow statement outside for/while loop.").into(),
        }
    }

    pub fn invalid_return(spanned: impl Spanned) -> Self {
        Self {
            span: spanned.span(),
            message: "Invalid return statement function.".into(),
        }
    }
}

impl Report for RuntimeError {
    fn report(&self, _source: &str) {
        eprint!("{}", self.message);
    }
}

impl Spanned for RuntimeError {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
