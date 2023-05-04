//! Grpc (tonic) specific errors

use std::{fmt, result};

use crate::errors::error_chain_fmt;

/// Wrappers around tonic errors
#[derive(thiserror::Error)]
pub enum ClientError {
    /// General transport error occurred (connection issues)
    #[error("Failed to connect to server")]
    Transport(#[from] tonic::transport::Error),
    /// Protocol level error occurred
    #[error("Failed to execute RPC request")]
    Status(#[from] tonic::Status),
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Newtype representing RPC client results
pub type ClientResult<T> = result::Result<T, ClientError>;
