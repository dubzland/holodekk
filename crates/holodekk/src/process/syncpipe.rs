//! Syncpipe processing for daemons.
//!
//! When daemonizing processes, the ultimate child becomes reparented to the init (pid 1) process.
//! Pipes are used to transfer the pid of the child process to the parent, so it can be continually
//! monitored.

use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};

use nix::{fcntl::OFlag, unistd::pipe2};
use serde::{Deserialize, Serialize};

use crate::errors::error_chain_fmt;

/// Errors encountered during start/stop/synchronication of daemons.
#[derive(thiserror::Error)]
pub enum Error {
    /// Error creating sync pipes
    #[error(transparent)]
    Create(#[from] nix::errno::Errno),
    /// Error (de)serializing sync message
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// Error during read/write operation
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Syncpipe methods result type
pub type Result<T> = std::result::Result<T, Error>;

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
/// use holodekk::process::syncpipe;
///
/// let (read_pipe, send_pipe) = syncpipe::create().unwrap();
/// ```
pub fn create() -> Result<(File, RawFd)> {
    let (parent_fd, child_fd) = pipe2(OFlag::empty())?;
    let sync_pipe = unsafe { File::from_raw_fd(parent_fd) };
    Ok((sync_pipe, child_fd))
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
/// use holodekk::process::syncpipe;
///
/// let (read_pipe, send_pipe) = syncpipe::create().unwrap();
/// syncpipe::write_pid(send_pipe, 123).unwrap();
/// ```
pub fn write_pid(sync_pipe_fd: RawFd, pid: i32) -> Result<()> {
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
/// use holodekk::process::syncpipe;
///
/// let (read_pipe, send_pipe) = syncpipe::create().unwrap();
/// let status = syncpipe::PidSyncMessage::new(123);
///
/// syncpipe::write_pid(send_pipe, 123);
///
/// let pid = syncpipe::read_pid(read_pipe).unwrap();
///
/// assert_eq!(pid, 123);
/// ```
pub fn read_pid(mut sync_pipe: File) -> Result<i32> {
    // wait for projector data via pipe
    let mut buf = [0; 256];
    let bytes_read = sync_pipe.read(&mut buf)?;

    // cleanup our end of the pipe
    drop(sync_pipe);

    // try and convert/parse the response
    let message = serde_json::from_slice::<PidSyncMessage>(&buf[0..bytes_read])?;
    Ok(message.pid)
}
