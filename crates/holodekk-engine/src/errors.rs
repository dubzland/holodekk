use std::{fmt, result};

use holodekk_utils::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("A Bollard error occured.")]
    BollardError(#[from] bollard::errors::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = result::Result<T, Error>;
