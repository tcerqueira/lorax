use thiserror::Error;

#[derive(Debug, Error)]
#[error("Pass error")]
pub struct PassError;
