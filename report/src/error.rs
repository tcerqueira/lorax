use std::process::{ExitCode, Termination};

use thiserror::Error;

pub mod lexing;
pub mod parsing;
pub mod pass;
pub mod runtime;

pub use lexing::Error as LexingError;
pub use parsing::Error as ParsingError;
pub use pass::Error as PassError;
pub use runtime::Error as RuntimeError;

#[derive(Debug, Error)]
pub enum Error {
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

impl Termination for Error {
    fn report(self) -> ExitCode {
        match self {
            Error::Parsing { .. } | Error::Lexing(_) | Error::Pass(_) => ExitCode::from(65),
            Error::Runtime(_) => ExitCode::from(70),
            Error::Other(_) => ExitCode::FAILURE,
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

impl From<parsing::Error> for Error {
    fn from(err: parsing::Error) -> Self {
        Self::Parsing(vec![err])
    }
}

impl From<Vec<parsing::Error>> for Error {
    fn from(errors: Vec<parsing::Error>) -> Self {
        Self::Parsing(errors)
    }
}

impl From<lexing::Error> for Error {
    fn from(err: lexing::Error) -> Self {
        Self::Lexing(vec![err])
    }
}

impl From<Vec<lexing::Error>> for Error {
    fn from(errors: Vec<lexing::Error>) -> Self {
        Self::Lexing(errors)
    }
}
