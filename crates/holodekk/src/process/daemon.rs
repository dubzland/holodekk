//! Daemon management made easy.

use std::fs::File;
use std::path::Path;
use std::process::Command;

use log::{debug, warn};
use nix::{
    sys::{
        signal::{kill, SIGINT, SIGKILL},
        wait::waitpid,
    },
    unistd::Pid,
};

use crate::errors::error_chain_fmt;
use crate::Paths;

/// Errors encountered while attenpting to spawn a background process.
#[derive(thiserror::Error)]
pub enum Error {
    /// Command execution failed (possibly incorrect arguments)
    #[error("failed to execute process command")]
    Command(std::process::ExitStatus),
    /// Command exited with an invalid status
    #[error("command returned bad execution status")]
    Execution(#[from] std::io::Error),
    /// Error during pidfile processing
    #[error(transparent)]
    Pidfile(#[from] super::pidfile::Error),
    /// Error during syncpipe processing
    #[error(transparent)]
    Syncpipe(#[from] super::syncpipe::Error),
    /// Termination requested, but no process is running with the given pid.
    #[error("termination requested, but no process exists matching pid")]
    NotRunning(i32),
    /// Something strange occurred during termination (more info in Errno).
    #[error("Unexpected error occurred")]
    Unexpected(#[from] nix::errno::Errno),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Daemon methods result type
pub type Result<T> = std::result::Result<T, Error>;

/// Retrieves the pid for a launched daemon, either via sync pipe or pid file.
///
/// # Errors
///
/// - `pidfile` does not exist
/// - insufficient permissions to read from `pidfile`
/// - `pidfile` did not contain a valid pid
pub fn get_pid<P: AsRef<Path>>(sync_pipe: File, pidfile: P) -> Result<i32> {
    let pid = match super::syncpipe::read_pid(sync_pipe) {
        Ok(pid) => pid,
        Err(err) => {
            warn!("Failed to read process pid from sync pipe: {err}");
            warn!("Trying pidfile {} ...", pidfile.as_ref().display());
            super::pidfile::read_pid(pidfile.as_ref())?
        }
    };
    Ok(pid)
}

/// Launches a background process from the provided [`Command`][`std::process::Command`] object.
///
/// On failure, the returned error will have the command output stored.
///
/// # Errors
///
/// - Malformed command
/// - Invalid command args
pub fn start<P>(paths: &Paths, mut command: Command, pidfile: P) -> Result<i32>
where
    P: AsRef<Path>,
{
    let (sync_pipe, child_fd) = super::syncpipe::create()?;

    command.arg("--data-root");
    command.arg(paths.data_root());
    command.arg("--exec-root");
    command.arg(paths.exec_root());
    command.arg("--bin-path");
    command.arg(paths.bin_root());
    command.arg("--sync-pipe");
    command.arg(child_fd.to_string());

    debug!("Spawning daemon: {:?}", command);

    let output = command.output()?;

    if output.status.success() {
        let pid = get_pid(sync_pipe, pidfile.as_ref())?;
        Ok(pid)
    } else {
        warn!("Unable to spawn process: {:?}", output.status);
        if let Ok(text) = std::str::from_utf8(&output.stdout) {
            warn!("=====================================================================");
            warn!("stdout:");
            warn!("{text}");
        }
        if let Ok(text) = std::str::from_utf8(&output.stderr) {
            warn!("=====================================================================");
            warn!("stderr:");
            warn!("{text}");
        }
        Err(Error::Command(output.status))
    }
}

/// Terminates a previously launched daemon using its pid (process id).
///
/// # Errors
///
/// - no process exists matching `pid`
/// - `pid` is not a process the running user has rights to
pub fn stop(pid: i32) -> Result<i32> {
    let pid = Pid::from_raw(pid);
    debug!("Terminating daemon with pid {}", pid);
    match kill(pid, None) {
        Ok(_) => {
            debug!("daemon active.  Attempting graceful shutdown ...");
            let mut count = 0;
            kill(pid, SIGINT)?;
            debug!("SIGTERM sent.  awaiting process termination ...");
            while count < 10 {
                if kill(pid, None).is_ok() {
                    count += 1;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                } else {
                    debug!("Process shutdown.  Termination complete");
                    return Ok(0);
                }
            }

            // if we're here, process is still running after 10 seconds.  Kill it for reals.
            kill(pid, SIGKILL)?;
            waitpid(pid, None)?;
            Ok(0)
        }
        Err(err) => {
            warn!(
                "Error encountered while checking the status of process: {}",
                err
            );
            Err(Error::NotRunning(pid.as_raw()))
        }
    }
}
