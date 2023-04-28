use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

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

use crate::config::HolodekkPaths;
use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum DaemonSyncError {
    #[error("failed to setup synchronization pipe")]
    Setup(#[from] nix::Error),
    #[error("failed to read process data")]
    Read(#[from] std::io::Error),
    #[error("failed to decode process data received from sync pipe")]
    Decode(#[from] serde_json::Error),
    #[error("failed to convert pidfile data to pid")]
    Parse(#[from] std::num::ParseIntError),
}

impl std::fmt::Debug for DaemonSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum DaemonTerminationError {
    #[error("termination requested, but no process exists matching pid")]
    NotRunning(i32),
    #[error("Unexpected error occurred")]
    Unexpected(#[from] nix::errno::Errno),
}

impl std::fmt::Debug for DaemonTerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum DaemonizeError {
    #[error("failed to execute process command")]
    Command(std::process::ExitStatus),
    #[error("command returned bad execution status")]
    Execution(#[from] std::io::Error),
    #[error("failed to synchronize with process")]
    Synchronize(#[from] DaemonSyncError),
}

impl std::fmt::Debug for DaemonizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PidSyncMessage {
    pid: i32,
}

impl PidSyncMessage {
    pub fn new(pid: i32) -> Self {
        Self { pid }
    }
}

pub fn setup_sync_pipe() -> std::result::Result<(File, RawFd), DaemonSyncError> {
    let (parent_fd, child_fd) = pipe2(OFlag::empty())?;
    let sync_pipe = unsafe { File::from_raw_fd(parent_fd) };
    Ok((sync_pipe, child_fd))
}

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

pub fn read_pid_from_pidfile<P: AsRef<Path>>(
    pidfile: P,
) -> std::result::Result<i32, DaemonSyncError> {
    let contents = fs::read_to_string(pidfile.as_ref())?;
    let pid: i32 = contents.parse()?;
    Ok(pid)
}

pub fn get_daemon_pid<P: AsRef<Path>>(
    sync_pipe: File,
    pidfile: P,
) -> std::result::Result<i32, DaemonSyncError> {
    let pid = match read_pid_from_sync_pipe(sync_pipe) {
        Ok(pid) => pid,
        Err(err) => {
            warn!("Failed to read process pid from sync pipe: {}", err);
            warn!("Trying pidfile {} ...", pidfile.as_ref().display());
            read_pid_from_pidfile(pidfile.as_ref())?
        }
    };
    Ok(pid)
}

pub fn daemonize<P>(
    paths: Arc<HolodekkPaths>,
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
        warn!("=====================================================================");
        warn!("stdout:");
        warn!("{}", std::str::from_utf8(&output.stdout).unwrap());
        warn!("=====================================================================");
        warn!("stderr:");
        warn!("{}", std::str::from_utf8(&output.stderr).unwrap());
        Err(DaemonizeError::Command(output.status))
    }
}

pub fn terminate_daemon(pid: i32) -> std::result::Result<i32, DaemonTerminationError> {
    debug!("Terminating daemon with pid {}", pid);
    match kill(Pid::from_raw(pid), None) {
        Ok(_) => {
            debug!("daemon active.  Attempting graceful shutdown ...");
            let mut count = 0;
            kill(Pid::from_raw(pid), SIGINT)?;
            debug!("SIGTERM sent.  awaiting process termination ...");
            while count < 10 {
                match kill(Pid::from_raw(pid), None) {
                    Ok(_) => {
                        count += 1;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    Err(_) => {
                        debug!("Process shutdown.  Termination complete");
                        return Ok(0);
                    }
                }
            }

            // if we're here, process is still running after 10 seconds.  Kill it for reals.
            kill(Pid::from_raw(pid), SIGKILL)?;
            waitpid(Pid::from_raw(pid), None)?;
            Ok(-1)
        }
        Err(err) => {
            warn!(
                "Error encountered while checking the status of process: {}",
                err
            );
            Err(DaemonTerminationError::NotRunning(pid))
        }
    }
}
