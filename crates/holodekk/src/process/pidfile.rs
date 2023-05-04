//! Pidfile processing for daemons.

use std::path::Path;

use crate::errors::error_chain_fmt;

/// Errors encountered during pidfile processing
#[derive(thiserror::Error)]
pub enum Error {
    /// Error during read/write of file
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Parse error on pidfile contents
    #[error(transparent)]
    Parse(#[from] std::num::ParseIntError),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Pidfile methods result type
pub type Result<T> = std::result::Result<T, Error>;

/// Writes the given pid to the specified pidfile.
///
/// # Errors
///
/// Permission denied to write to/create pidfile.
///
/// # Examples
///
/// ```rust
/// use holodekk::process::pidfile;
///
/// pidfile::write_pid("/tmp/test.pid", std::process::id() as i32).unwrap();
/// ```
pub fn write_pid<P: AsRef<Path>>(pidfile: P, pid: i32) -> Result<()> {
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
/// use holodekk::process::pidfile;
///
/// pidfile::write_pid("/tmp/test.pid", std::process::id() as i32).unwrap();
///
/// let pid = pidfile::read_pid("/tmp/test.pid").unwrap();
///
/// assert_eq!(pid, std::process::id() as i32);
/// ```
pub fn read_pid<P: AsRef<Path>>(pidfile: P) -> Result<i32> {
    let contents = std::fs::read_to_string(pidfile.as_ref())?;
    let pid: i32 = contents.parse()?;
    Ok(pid)
}
