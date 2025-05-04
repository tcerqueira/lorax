use thiserror::Error;

use crate::{
    lexing::error::LexingError, parsing::error::ParsingError, runtime::error::RuntimeError,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Usage: rlox [script]")]
    Cli,
    #[error("{n} errors:\n{list}", n = .0.len(), list = display_error_list(.0))]
    Lexing(Vec<LexingError>),
    #[error("{n} errors:\n{list}", n = .0.len(), list = display_error_list(.0))]
    Parsing(Vec<ParsingError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

fn display_error_list(errors: &[impl ToString]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

impl From<ParsingError> for Error {
    fn from(err: ParsingError) -> Self {
        Self::Parsing(vec![err])
    }
}

impl From<Vec<ParsingError>> for Error {
    fn from(errors: Vec<ParsingError>) -> Self {
        Self::Parsing(errors)
    }
}

impl From<LexingError> for Error {
    fn from(err: LexingError) -> Self {
        Self::Lexing(vec![err])
    }
}

impl From<Vec<LexingError>> for Error {
    fn from(errors: Vec<LexingError>) -> Self {
        Self::Lexing(errors)
    }
}
