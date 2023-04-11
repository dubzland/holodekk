mod handle;

use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use log::{debug, info, warn};
use nix::{
    fcntl::OFlag,
    sys::signal::{kill, SIGINT},
    unistd::{pipe2, Pid},
};
use serde::Deserialize;
use uuid::Uuid;

use holodekk::errors::error_chain_fmt;
use holodekk::utils::ConnectionInfo;

pub use handle::*;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Error launching projector: {0:?}")]
    LaunchError(std::process::ExitStatus),
    #[error("Error synchronizing with projector process")]
    SyncError(#[from] serde_json::Error),
    #[error("Failed to connect to the projector.")]
    Connect(#[from] tonic::transport::Error),
    #[error("Failed to execute RPC call.")]
    Rpc(#[from] tonic::Status),
    #[error("Failed to shutdown server gracefully")]
    Shutdown,
    #[error("IO error occurred")]
    Io(#[from] std::io::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Deserialize)]
struct MessageProjectorPidParent {
    pid: i32,
}

#[derive(Clone, Debug)]
pub struct Projector {
    pub id: Uuid,
    pub fleet: String,
    pub namespace: String,
    pub pidfile: PathBuf,
    pub uhura_address: ConnectionInfo,
    pub projector_address: ConnectionInfo,
    pub pid: Pid,
}

impl Projector {
    pub fn new(
        fleet: &str,
        namespace: &str,
        pidfile: &PathBuf,
        uhura_address: ConnectionInfo,
        projector_address: ConnectionInfo,
        pid: Pid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            fleet: fleet.to_string(),
            namespace: namespace.to_string(),
            pidfile: pidfile.to_owned(),
            uhura_address,
            projector_address,
            pid,
        }
    }

    pub fn handle(&self) -> ProjectorHandle {
        ProjectorHandle::new(&self.id, &self.fleet, &self.namespace, &self.uhura_address)
    }

    pub fn spawn<P>(
        fleet: &str,
        namespace: &str,
        root_path: P,
        bin_path: P,
        uhura_port: Option<u16>,
        projector_port: Option<u16>,
    ) -> std::result::Result<Projector, Error>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        // Setup a pipe so we can be notified when the projector is fully up
        let (parent_fd, child_fd) = pipe2(OFlag::empty()).unwrap();
        let mut sync_pipe = unsafe { File::from_raw_fd(parent_fd) };

        // Ensure the root directory exists
        if !root_path.as_ref().exists() {
            fs::create_dir_all(&root_path)?;
        }

        let mut pidfile = root_path.as_ref().to_path_buf();
        pidfile.push("uhura.pid");

        let mut uhura = bin_path.as_ref().to_path_buf();
        uhura.push("uhura");

        let mut command = Command::new(uhura);
        command.arg("--fleet");
        command.arg(fleet);
        command.arg("--namespace");
        command.arg(namespace);
        command.arg("--pidfile");
        command.arg(&pidfile);
        command.arg("--sync-pipe");
        command.arg(child_fd.to_string());

        let uhura_listener = if let Some(port) = uhura_port {
            command.arg("--uhura-port");
            command.arg(port.to_string());
            ConnectionInfo::tcp(&port, None)
        } else {
            let mut socket: PathBuf = root_path.as_ref().to_owned();
            socket.push("uhura.sock");
            command.arg("--uhura-socket");
            command.arg(&socket);
            ConnectionInfo::unix(&socket)
        };

        let projector_listener = if let Some(port) = projector_port {
            command.arg("--projector-port");
            command.arg(port.to_string());
            ConnectionInfo::tcp(&port, None)
        } else {
            let mut socket: PathBuf = root_path.into();
            socket.push("projector.sock");
            command.arg("--projector-socket");
            command.arg(&socket);
            ConnectionInfo::unix(&socket)
        };

        info!("Launching uhura with: {:?}", command);
        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env_clear()
            .status()?;

        if status.success() {
            let mut buf = [0; 256];
            let bytes_read = sync_pipe.read(&mut buf)?;
            let msg: MessageProjectorPidParent = serde_json::from_slice(&buf[0..bytes_read])?;
            let p = Self::new(
                fleet,
                namespace,
                &pidfile,
                uhura_listener,
                projector_listener,
                Pid::from_raw(msg.pid),
            );
            debug!("Uhura spawned with pid: {}", p.pid);
            drop(sync_pipe);
            Ok(p)
        } else {
            warn!("failed to launch uhura");
            Err(Error::LaunchError(status))
        }
    }
}

impl Drop for Projector {
    fn drop(&mut self) {
        // TODO: check to see if uhura is still running before blindly killing it
        match kill(self.pid, SIGINT) {
            Ok(_) => debug!(
                "stopped uhura running for namespace {} with pid {}",
                self.namespace, self.pid
            ),
            Err(err) => warn!(
                "failed stop uhura running for namespace {} with pid {}: {}",
                self.namespace, self.pid, err
            ),
        };
    }
}
