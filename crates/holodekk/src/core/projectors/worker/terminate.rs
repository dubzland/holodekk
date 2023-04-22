use log::{debug, trace};

use crate::config::HolodekkConfig;
use crate::core::projectors::entities::{ProjectorEntity, ProjectorStatus};
use crate::errors::error_chain_fmt;
use crate::process::{terminate_daemon, ProcessTerminationError};

use super::ProjectorsWorker;

/// Error encountered during Projector termination
#[derive(thiserror::Error)]
pub enum TerminationError {
    #[error("error encountered while terminating projector daemon")]
    Termination(#[from] ProcessTerminationError),
    #[error("failed to cleanup projector directory")]
    Cleanup(#[from] std::io::Error),
}

impl std::fmt::Debug for TerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl<C> ProjectorsWorker<C>
where
    C: HolodekkConfig,
{
    /// Terminate a projector running in the background
    ///
    /// Sends a SIGTERM to the projector process and waits for termination.
    ///
    /// # Arguments
    ///
    /// `projector` - [ProjectorEntity] being shutdown
    pub async fn terminate(
        &self,
        projector: &ProjectorEntity,
    ) -> std::result::Result<(), TerminationError> {
        trace!("ProjectorsWorker::terminate({:?})", projector);
        if let ProjectorStatus::Running(pid) = projector.status() {
            terminate_daemon(pid as i32)?;

            std::fs::remove_dir_all(projector.root())?;
            debug!("Projector cleanup complete.");
        }

        Ok(())
    }
}
