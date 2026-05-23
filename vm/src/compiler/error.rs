use report::error::{LexingError, ParsingError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Lexing(#[from] LexingError),
    #[error(transparent)]
    Parsing(#[from] ParsingError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<CompileError> for report::Error {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::Lexing(e) => e.into(),
            CompileError::Parsing(e) => e.into(),
            CompileError::Other(e) => e.into(),
        }
    }
}
