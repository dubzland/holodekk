use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

use log::warn;
use nix::{
    fcntl::OFlag,
    sys::{
        signal::{kill, SIGINT, SIGKILL},
        wait::{waitpid, WaitPidFlag, WaitStatus},
    },
    unistd::{pipe2, Pid},
};
use serde::{Deserialize, Serialize};

use crate::config::HolodekkConfig;
use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum ProcessSyncError {
    #[error("failed to setup synchronization pipe")]
    Setup(#[from] nix::Error),
    #[error("failed to read process data")]
    Read(#[from] std::io::Error),
    #[error("failed to decode process data received from sync pipe")]
    Decode(#[from] serde_json::Error),
    #[error("failed to convert pidfile data to pid")]
    Parse(#[from] std::num::ParseIntError),
}

impl std::fmt::Debug for ProcessSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum ProcessTerminationError {
    #[error("termination requested, but no process exists matching pid")]
    NotRunning(i32),
    #[error("Unexpected error occurred")]
    Unexpected(#[from] nix::errno::Errno),
}

impl std::fmt::Debug for ProcessTerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum DaemonizeError {
    #[error("failed to execute process command")]
    Execute(#[from] std::io::Error),
    #[error("failed to synchronize with process")]
    Synchronize(#[from] ProcessSyncError),
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

pub fn setup_sync_pipe() -> std::result::Result<(File, RawFd), ProcessSyncError> {
    let (parent_fd, child_fd) = pipe2(OFlag::empty())?;
    let sync_pipe = unsafe { File::from_raw_fd(parent_fd) };
    Ok((sync_pipe, child_fd))
}

pub fn read_pid_from_sync_pipe(mut sync_pipe: File) -> std::result::Result<i32, ProcessSyncError> {
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
) -> std::result::Result<i32, ProcessSyncError> {
    let contents = fs::read_to_string(pidfile.as_ref())?;
    let pid: i32 = contents.parse()?;
    Ok(pid)
}

pub fn get_daemon_pid<P: AsRef<Path>>(
    sync_pipe: File,
    pidfile: P,
) -> std::result::Result<i32, ProcessSyncError> {
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

pub fn daemonize<C, P>(
    config: Arc<C>,
    mut command: Command,
    pidfile: P,
) -> std::result::Result<i32, DaemonizeError>
where
    C: HolodekkConfig,
    P: AsRef<Path>,
{
    let (sync_pipe, child_fd) = setup_sync_pipe()?;

    command.arg("--data-root");
    command.arg(config.data_root());
    command.arg("--exec-root");
    command.arg(config.exec_root());
    command.arg("--bin-path123");
    command.arg(config.bin_root());
    command.arg("--sync-pipe");
    command.arg(child_fd.to_string());

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let status = command.status()?;

    if status.success() {
        let pid = get_daemon_pid(sync_pipe, pidfile.as_ref())?;
        Ok(pid)
    } else {
        todo!()
    }
}

pub fn terminate_daemon(pid: i32) -> std::result::Result<i32, ProcessTerminationError> {
    match kill(Pid::from_raw(pid), None) {
        Ok(_) => {
            kill(Pid::from_raw(pid), SIGINT)?;
            let mut count = 0;
            while count < 10 {
                match waitpid(Pid::from_raw(pid), Some(WaitPidFlag::WNOHANG))? {
                    WaitStatus::Exited(_, code) => {
                        return Ok(code);
                    }
                    WaitStatus::Signaled(..) => {
                        return Ok(-1);
                    }
                    _ => {
                        // process still active.  sleep and try again
                        count += 1;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
            }

            // if we're here, process is still running after 10 seconds.  Kill it for reals.
            kill(Pid::from_raw(pid), SIGKILL)?;
            waitpid(Pid::from_raw(pid), None)?;
            Ok(-1)
        }
        Err(_) => Err(ProcessTerminationError::NotRunning(pid)),
    }
}
