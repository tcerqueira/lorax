use thiserror::Error;

#[derive(Debug, Error)]
#[error("Pass error")]
#[expect(dead_code)]
pub struct PassError;
