use std::process::{ExitCode, Termination};

use rlox_report::Error as InterpreterError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Usage: rlox [script]")]
    Cli,
    #[error(transparent)]
    Interpreter(#[from] InterpreterError),
}

impl Termination for Error {
    fn report(self) -> ExitCode {
        match self {
            Error::Cli => ExitCode::from(64),
            Error::Interpreter(err) => err.report(),
        }
    }
}
