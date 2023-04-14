use log::{debug, warn};
use nix::sys::signal::{kill, SIGINT};
use nix::unistd::Pid;

use crate::core::entities::{Subroutine, SubroutineInstance, SubroutineStatus};

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ShutdownError {
    #[error("Failed to send SIGINT to subroutine process")]
    Kill(#[from] nix::Error),
}

pub fn shutdown_subroutine(
    instance: SubroutineInstance,
    subroutine: Subroutine,
) -> std::result::Result<(), ShutdownError> {
    // TODO: check to see if the subroutine is still running before blindly killing it
    match instance.status {
        SubroutineStatus::Running(pid) => match kill(Pid::from_raw(pid as i32), SIGINT) {
            Ok(_) => {
                debug!(
                    "stopped subroutine {} running in namespace {} with pid {}",
                    subroutine.name, instance.namespace, pid
                );
                Ok(())
            }
            Err(err) => {
                warn!(
                    "failed stop subroutine {} running in namespace {} with pid {}: {}",
                    subroutine.name, instance.namespace, pid, err
                );
                Err(ShutdownError::from(err))
            }
        },
        status => {
            warn!(
                "Requested shutdown for subroutine that is not running: {:?}",
                status
            );
            Ok(())
        }
    }
}
