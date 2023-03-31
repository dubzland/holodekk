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
    #[error("Projector does not exist: {0}")]
    InvalidProjector(crate::ProjectorHandle),
    #[error("Failed to start projector.")]
    ProjectorError(#[from] holodekk_projector::Error),
    #[error("IO error.")]
    Io(#[from] std::io::Error),
    // #[error("Invalid input.")]
    // ParseInt(#[from] ParseIntError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = result::Result<T, Error>;
