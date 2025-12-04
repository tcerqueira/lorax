use std::process::{ExitCode, Termination};

use rlox_tree_walk::error::TreeWalkError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Usage: rlox [script]")]
    Cli,
    #[error(transparent)]
    TreeWalkInterpreter(#[from] TreeWalkError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Termination for Error {
    fn report(self) -> ExitCode {
        match self {
            Error::Cli => ExitCode::from(64),
            Error::TreeWalkInterpreter(err) => err.report(),
            Error::Other(_) => ExitCode::FAILURE,
        }
    }
}
