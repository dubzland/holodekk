//! Utilities to spawn external programs in the background.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::{
    fd::AsRawFd,
    unix::io::{FromRawFd, RawFd},
};
use std::path::Path;
use std::process::Command;

use log::{debug, warn};
use nix::{
    fcntl::OFlag,
    sys::{
        signal::{kill, SIGINT, SIGKILL},
        wait::waitpid,
    },
    unistd::{pipe2, Pid},
};
use serde::{Deserialize, Serialize};

use crate::errors::error_chain_fmt;
use crate::Paths;

/// Errors encountered while attenpting to spawn a background process.
#[derive(thiserror::Error)]
pub enum DaemonizeError {
    /// Command execution failed (possibly incorrect arguments)
    #[error("failed to execute process command")]
    Command(std::process::ExitStatus),
    /// Command exited with an invalid status
    #[error("command returned bad execution status")]
    Execution(#[from] std::io::Error),
    /// Failed to receive process info via sync pipe
    #[error("failed to synchronize with process")]
    Synchronize(#[from] DaemonSyncError),
}

impl std::fmt::Debug for DaemonizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Errors encountered while attempting to receive process information from daemon.
#[derive(thiserror::Error)]
pub enum DaemonSyncError {
    /// Failed to setup the sync pipe
    #[error("failed to setup synchronization pipe")]
    Setup(#[from] nix::Error),
    /// Failed to read from sync pipe
    #[error("failed to read process data")]
    Read(#[from] std::io::Error),
    /// Failed to decode data received via sync pipe
    #[error("failed to decode process data received from sync pipe")]
    Decode(#[from] serde_json::Error),
    /// Failed to convert pid received from process to number
    #[error("failed to convert pidfile data to pid")]
    Parse(#[from] std::num::ParseIntError),
}

impl std::fmt::Debug for DaemonSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Errors enountered while attempting to terminate a spawned daemon.
#[derive(thiserror::Error)]
pub enum DaemonTerminationError {
    /// Termination requested, but no process is running with the given pid.
    #[error("termination requested, but no process exists matching pid")]
    NotRunning(i32),
    /// Something strange occurred during termination (more info in Errno).
    #[error("Unexpected error occurred")]
    Unexpected(#[from] nix::errno::Errno),
}

impl std::fmt::Debug for DaemonTerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Structure used to share pid information between daemon and parent.
#[derive(Debug, Deserialize, Serialize)]
pub struct PidSyncMessage {
    pid: i32,
}

impl PidSyncMessage {
    /// Construct a new [`PidSyncMessage`] from the given `pid`.
    #[must_use]
    pub fn new(pid: i32) -> Self {
        Self { pid }
    }
}

/// Creates a pipe pair used to exchange information between daemon and parent.
///
/// # Errors
///
/// Most likely error would be hitting the limit on open file descriptors.
///
/// # Examples
///
/// ```rust
/// use holodekk::utils::process;
///
/// let (read_pipe, send_pipe) = process::setup_sync_pipe().unwrap();
/// ```
pub fn setup_sync_pipe() -> std::result::Result<(File, RawFd), DaemonSyncError> {
    let (parent_fd, child_fd) = pipe2(OFlag::empty())?;
    let sync_pipe = unsafe { File::from_raw_fd(parent_fd.as_raw_fd()) };
    Ok((sync_pipe, child_fd.as_raw_fd()))
}

/// Writes process (pid) information to the child end of a sync pipe.
///
/// # Errors
///
/// Most likely caused by the parent disappearing before the process data could be written.
///
/// # Example
///
/// ```rust
/// use holodekk::utils::process;
///
/// let (read_pipe, send_pipe) = process::setup_sync_pipe().unwrap();
/// process::write_pid_to_sync_pipe(send_pipe, 123).unwrap();
/// ```
pub fn write_pid_to_sync_pipe(
    sync_pipe_fd: RawFd,
    pid: i32,
) -> std::result::Result<(), DaemonSyncError> {
    let status = PidSyncMessage::new(pid);
    let msg = serde_json::to_vec(&status)?;
    let mut sync_pipe = unsafe { File::from_raw_fd(sync_pipe_fd) };
    sync_pipe.write_all(&msg)?;
    Ok(())
}

/// Reads process (pid) information from the parent end of a sync pipe.
///
/// # Errors
///
/// Either the pipe was closed somehow, or the data was garbled in transmission and could not be
/// decoded.
///
/// # Example
///
/// ```rust
/// use holodekk::utils::process;
///
/// let (read_pipe, send_pipe) = process::setup_sync_pipe().unwrap();
/// let status = process::PidSyncMessage::new(123);
///
/// process::write_pid_to_sync_pipe(send_pipe, 123);
///
/// let pid = process::read_pid_from_sync_pipe(read_pipe).unwrap();
///
/// assert_eq!(pid, 123);
/// ```
pub fn read_pid_from_sync_pipe(mut sync_pipe: File) -> std::result::Result<i32, DaemonSyncError> {
    // wait for projector data via pipe
    let mut buf = [0; 256];
    let bytes_read = sync_pipe.read(&mut buf)?;

    // cleanup our end of the pipe
    drop(sync_pipe);

    // try and convert/parse the response
    let message = serde_json::from_slice::<PidSyncMessage>(&buf[0..bytes_read])?;
    Ok(message.pid)
}

/// Writes the given pid to the specified pidfile.
///
/// # Errors
///
/// Permission denied to write to/create pidfile.
///
/// # Examples
///
/// ```rust
/// use holodekk::utils::process;
///
/// process::write_pid_to_pid_file("/tmp/test.pid", std::process::id()).unwrap();
/// ```
pub fn write_pid_to_pid_file<P: AsRef<Path>>(
    pidfile: P,
    pid: i32,
) -> std::result::Result<(), DaemonSyncError> {
    std::fs::write(pidfile.as_ref(), format!("{pid}"))?;
    Ok(())
}

/// Reads a pid from the given pidfile.
///
/// # Errors
///
/// - Permission denied to read from pidfile
/// - Pidfile does not exist
/// - Pidfile did not contain a valid process id
///
/// # Examples
///
/// ```rust
/// use holodekk::utils::process;
///
/// process::write_pid_to_pid_file("/tmp/test.pid", std::process::id()).unwrap();
///
/// let pid = process::read_pid_from_pid_file("/tmp/test.pid").unwrap();
///
/// assert_eq!(pid, std::process::id());
/// ```
pub fn read_pid_from_pid_file<P: AsRef<Path>>(
    pidfile: P,
) -> std::result::Result<i32, DaemonSyncError> {
    let contents = fs::read_to_string(pidfile.as_ref())?;
    let pid: i32 = contents.parse()?;
    Ok(pid)
}

/// Retrieves the pid for a launched daemon, either via sync pipe or pid file.
///
/// # Errors
///
/// - `pidfile` does not exist
/// - insufficient permissions to read from `pidfile`
/// - `pidfile` did not contain a valid pid
pub fn get_daemon_pid<P: AsRef<Path>>(
    sync_pipe: File,
    pidfile: P,
) -> std::result::Result<i32, DaemonSyncError> {
    let pid = match read_pid_from_sync_pipe(sync_pipe) {
        Ok(pid) => pid,
        Err(err) => {
            warn!("Failed to read process pid from sync pipe: {}", err);
            warn!("Trying pidfile {} ...", pidfile.as_ref().display());
            read_pid_from_pid_file(pidfile.as_ref())?
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
pub fn daemonize<P>(
    paths: &Paths,
    mut command: Command,
    pidfile: P,
) -> std::result::Result<i32, DaemonizeError>
where
    P: AsRef<Path>,
{
    let (sync_pipe, child_fd) = setup_sync_pipe()?;

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
        let pid = get_daemon_pid(sync_pipe, pidfile.as_ref())?;
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
        Err(DaemonizeError::Command(output.status))
    }
}

/// Terminates a previously launched daemon using its pid (process id).
///
/// # Errors
///
/// - no process exists matching `pid`
/// - `pid` is not a process the running user has rights to
pub fn terminate_daemon(pid: i32) -> std::result::Result<i32, DaemonTerminationError> {
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
            Err(DaemonTerminationError::NotRunning(pid.as_raw()))
        }
    }
}
