use crate::error::Error;

pub mod error;
pub mod test_utils;

pub type Result<T> = ::std::result::Result<T, Error>;
