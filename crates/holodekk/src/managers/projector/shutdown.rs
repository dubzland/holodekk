use std::sync::Arc;

use log::{debug, warn};
use nix::sys::signal::{kill, SIGINT};

use crate::config::HolodekkConfig;
use crate::core::entities::Projector;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ShutdownError {
    #[error("Failed to send SIGINT to projector process")]
    Kill(#[from] nix::Error),
}

pub fn shutdown_projector(
    _config: Arc<HolodekkConfig>,
    projector: Projector,
) -> std::result::Result<(), ShutdownError> {
    // TODO: check to see if uhura is still running before blindly killing it
    match kill(projector.pid, SIGINT) {
        Ok(_) => {
            debug!(
                "stopped uhura running for namespace {} with pid {}",
                projector.namespace, projector.pid
            );
            Ok(())
        }
        Err(err) => {
            warn!(
                "failed stop uhura running for namespace {} with pid {}: {}",
                projector.namespace, projector.pid, err
            );
            Err(ShutdownError::from(err))
        }
    }
}
