use thiserror::Error;

use crate::runtime::{error::RuntimeError, object::Object};

#[derive(Debug, Error)]
pub enum ControlFlow {
    #[error(transparent)]
    Error(#[from] RuntimeError),
    #[error("Return({0})")]
    Return(Object),
    #[error("Break")]
    Break,
    #[error("Continue")]
    Continue,
}
