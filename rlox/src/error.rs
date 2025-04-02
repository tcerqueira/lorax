use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Usage: rlox [script]")]
    Cli,
    #[error("{n} errors:\n{list}", n = .0.len(), list = display_compile_errors(.0))]
    Compile(Vec<CompileError>),
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
#[error("[line {line}] Error{span}: {message}")]
pub struct CompileError {
    pub line: u32,
    pub span: String,
    pub message: String,
}
