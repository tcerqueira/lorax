use std::fmt::Display;

use thiserror::Error;

use crate::{parser::expr::Expr, tokens::Token};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Usage: rlox [script]")]
    Cli,
    #[error("{n} errors:\n{list}", n = .0.len(), list = display_compile_errors(.0))]
    Compile(Vec<CompileError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

fn display_compile_errors(errors: &[CompileError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

impl From<CompileError> for Error {
    fn from(err: CompileError) -> Self {
        Self::Compile(vec![err])
    }
}

impl From<Vec<CompileError>> for Error {
    fn from(errors: Vec<CompileError>) -> Self {
        Self::Compile(errors)
    }
}

#[derive(Debug, Error)]
#[error("[line {line}] Error <{span}>: {message}")]
pub struct CompileError {
    pub line: u32,
    pub span: Box<str>,
    pub message: Box<str>,
}

impl CompileError {
    #[expect(dead_code)]
    pub fn custom(src: &str, token: &Token, message: impl Display) -> Self {
        Self {
            line: token.span.line_start,
            span: src[token.span.start..token.span.end].into(),
            message: format!("{message}").into(),
        }
    }

    pub fn expected(src: &str, expected: impl Display, found: &Token) -> Self {
        Self {
            line: found.span.line_start,
            span: src[found.span.start..found.span.end].into(),
            message: format!("Expected '{}', found '{}'", expected, found.ty).into(),
        }
    }
}

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
            span: src[span.start..span.end].into(),
            message: format!("{message}").into(),
        }
    }
}
