use log::{debug, trace};

use crate::config::HolodekkConfig;
use crate::core::subroutines::{
    entities::{SubroutineEntity, SubroutineStatus},
    SubroutinePaths,
};
use crate::errors::error_chain_fmt;
use crate::process::{terminate_daemon, ProcessTerminationError};

use super::SubroutinesWorker;

/// Error encountered during Subroutine termination
#[derive(thiserror::Error)]
pub enum TerminationError {
    #[error("error encountered while terminating subroutine daemon")]
    Termination(#[from] ProcessTerminationError),
    #[error("failed to cleanup subroutine directory")]
    Cleanup(#[from] std::io::Error),
}

impl std::fmt::Debug for TerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl<C> SubroutinesWorker<C>
where
    C: HolodekkConfig,
{
    /// Terminate a subroutine running in the background
    ///
    /// Sends a SIGTERM to the subroutine process and waits for termination.
    ///
    /// # Arguments
    ///
    /// `projector` - [ProjectorEntity] being shutdown
    pub async fn terminate(
        &self,
        subroutine: &SubroutineEntity,
    ) -> std::result::Result<(), TerminationError> {
        trace!("SubroutinesWorker::terminate({:?})", subroutine);
        if let SubroutineStatus::Running(pid) = subroutine.status() {
            terminate_daemon(pid as i32)?;

            let paths = SubroutinePaths::build(self.config.clone(), subroutine);
            std::fs::remove_dir_all(paths.root())?;
            debug!("Subroutine cleanup complete.");
        }

        Ok(())
    }
}
