use report::error::RuntimeError;
use thiserror::Error;

use crate::enconding::DecodeError;

#[derive(Debug, Error)]
pub enum VirtualMachineError {
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
