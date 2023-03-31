use std::{error, fmt, result};

fn error_chain_fmt(e: &impl error::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Error launching projector: {0:?}")]
    LaunchError(std::process::ExitStatus),
    #[error("Error synchronizing with projector process")]
    SyncError(#[from] serde_json::Error),
    #[error("Failed to connect to the projector.")]
    Connect(#[from] tonic::transport::Error),
    #[error("Failed to execute RPC call.")]
    Rpc(#[from] tonic::Status),
    #[error("Failed to shutdown server gracefully")]
    Shutdown,
    #[error("IO error occurred")]
    Io(#[from] std::io::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = result::Result<T, Error>;
