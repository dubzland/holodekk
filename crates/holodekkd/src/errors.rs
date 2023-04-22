use std::fmt;

use uuid::Uuid;

use holodekk::core::projectors::ProjectorsError;
use holodekk::errors::{error_chain_fmt, grpc::GrpcClientError};

#[derive(thiserror::Error)]
pub enum HolodekkError {
    #[error("Build error")]
    BuildFailed,
    #[error("Failed to detect runtime environment.")]
    RuntimeDetectFailed,
    #[error("Image not found.")]
    ImageNotFound(String),
    #[error("Tag not found.")]
    TagNotFound(String),
    #[error("Attempt to access handle on uninitialized server.")]
    Uninitialized,
    #[error("Invalid address supplied: {0}.")]
    InvalidAddress(String),
    #[error("Invalid engine supplied: {0}.")]
    InvalidEngine(String),
    #[error("Projector does not exist: {id}")]
    InvalidProjector { id: Uuid },
    #[error("IO error.")]
    Io(#[from] std::io::Error),
    #[error("RPC Client error")]
    RpcClient(#[from] GrpcClientError),
    #[error("Projector error")]
    ProjectorsError(#[from] ProjectorsError),
    // #[error("Subroutine error")]
    // SubroutineError(#[from] SubroutinesError),
    // #[error("Subroutine definitions error")]
    // SubroutineDefinitionsError(#[from] SubroutineDefinitionsError),
    // #[error("Invalid input.")]
    // ParseInt(#[from] ParseIntError),
}

impl fmt::Debug for HolodekkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}