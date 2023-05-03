use std::{fmt, result};

use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum ClientError {
    #[error("Failed to connect to server")]
    Transport(#[from] tonic::transport::Error),
    #[error("Failed to execute RPC request")]
    Status(#[from] tonic::Status),
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type ClientResult<T> = result::Result<T, ClientError>;
