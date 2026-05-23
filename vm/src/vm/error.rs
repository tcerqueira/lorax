use report::error::RuntimeError;
use thiserror::Error;

use crate::{enconding::DecodeError, opcode::OpDecodeError};

#[derive(Debug, Error)]
pub enum VirtualMachineError {
    #[error(transparent)]
    Decode(#[from] DecodeError<OpDecodeError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
