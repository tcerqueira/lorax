use std::process::{ExitCode, Termination};

use rlox_lexer::error::LexingError;
use thiserror::Error;

use crate::{parsing::error::ParsingError, passes::error::PassError, runtime::error::RuntimeError};

#[derive(Debug, Error)]
pub enum TreeWalkError {
    #[error("{n} errors:\n{list}", n = .0.len(), list = display_error_list(.0))]
    Lexing(Vec<LexingError>),
    #[error("{n} errors:\n{list}", n = .0.len(), list = display_error_list(.0))]
    Parsing(Vec<ParsingError>),
    #[error(transparent)]
    Pass(#[from] PassError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Termination for TreeWalkError {
    fn report(self) -> ExitCode {
        match self {
            TreeWalkError::Parsing { .. } | TreeWalkError::Lexing(_) | TreeWalkError::Pass(_) => {
                ExitCode::from(65)
            }
            TreeWalkError::Runtime(_) => ExitCode::from(70),
            TreeWalkError::Other(_) => ExitCode::FAILURE,
        }
    }
}

fn display_error_list(errors: &[impl ToString]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

impl From<ParsingError> for TreeWalkError {
    fn from(err: ParsingError) -> Self {
        Self::Parsing(vec![err])
    }
}

impl From<Vec<ParsingError>> for TreeWalkError {
    fn from(errors: Vec<ParsingError>) -> Self {
        Self::Parsing(errors)
    }
}

impl From<LexingError> for TreeWalkError {
    fn from(err: LexingError) -> Self {
        Self::Lexing(vec![err])
    }
}

impl From<Vec<LexingError>> for TreeWalkError {
    fn from(errors: Vec<LexingError>) -> Self {
        Self::Lexing(errors)
    }
}
