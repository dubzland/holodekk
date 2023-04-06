use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::net::Ipv4Addr;
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

use crate::errors::error_chain_fmt;

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

#[derive(Clone, Debug)]
pub struct Listener {
    port: Option<u16>,
    address: Option<Ipv4Addr>,
    socket: Option<PathBuf>,
}

impl Listener {
    pub fn new(port: Option<&u16>, address: Option<&Ipv4Addr>, socket: Option<&PathBuf>) -> Self {
        Self {
            port: port.map(|x| x.to_owned()),
            address: address.map(|x| x.to_owned()),
            socket: socket.map(|x| x.to_owned()),
        }
    }

    pub fn port(&self) -> Option<&u16> {
        self.port.as_ref()
    }

    pub fn address(&self) -> Option<&Ipv4Addr> {
        self.address.as_ref()
    }

    pub fn socket(&self) -> Option<&PathBuf> {
        self.socket.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ProjectorHandle {
    pub id: Uuid,
    pub namespace: String,
    pub pidfile: PathBuf,
    pub pid: Pid,
    pub uhura_listener: Listener,
    pub projector_listener: Listener,
}

impl fmt::Display for ProjectorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Deserialize)]
struct MessageProjectorPidParent {
    pid: i32,
    projector_port: Option<u16>,
    projector_address: Option<Ipv4Addr>,
    projector_socket: Option<PathBuf>,
    uhura_port: Option<u16>,
    uhura_address: Option<Ipv4Addr>,
    uhura_socket: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Projector {
    handle: ProjectorHandle,
}

impl Projector {
    pub fn new(
        namespace: &str,
        pidfile: &PathBuf,
        uhura_listener: Listener,
        projector_listener: Listener,
        pid: Pid,
    ) -> Self {
        Self {
            handle: ProjectorHandle {
                id: Uuid::new_v4(),
                namespace: namespace.to_string(),
                pidfile: pidfile.to_owned(),
                uhura_listener,
                projector_listener,
                pid,
            },
        }
    }

    pub fn handle(&self) -> ProjectorHandle {
        self.handle.clone()
    }

    pub fn id(&self) -> &Uuid {
        &self.handle.id
    }

    pub fn namespace(&self) -> &str {
        &self.handle.namespace
    }

    pub fn pidfile(&self) -> &Path {
        &self.handle.pidfile
    }

    pub fn pid(&self) -> &Pid {
        &self.handle.pid
    }

    pub fn uhura_listener(&self) -> &Listener {
        &self.handle.uhura_listener
    }

    pub fn projector_listener(&self) -> &Listener {
        &self.handle.projector_listener
    }

    pub fn spawn<P: AsRef<Path>>(
        namespace: &str,
        root_path: P,
        bin_path: P,
        uhura_port: Option<u16>,
        projector_port: Option<u16>,
    ) -> std::result::Result<Projector, Error> {
        debug!("inside spawn()");
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
        command.arg("--namespace");
        command.arg(namespace);
        command.arg("--pidfile");
        command.arg(&pidfile);
        command.arg("--sync-pipe");
        command.arg(child_fd.to_string());

        if let Some(port) = uhura_port {
            command.arg("--uhura-port");
            command.arg(port.to_string());
        } else {
            let mut socket = root_path.as_ref().to_path_buf();
            socket.push("uhura.sock");
            command.arg("--uhura-socket");
            command.arg(socket);
        }

        if let Some(port) = projector_port {
            command.arg("--projector-port");
            command.arg(port.to_string());
        } else {
            let mut socket = root_path.as_ref().to_path_buf();
            socket.push("projector.sock");
            command.arg("--projector-socket");
            command.arg(socket);
        }

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
            let uhura_listener = Listener::new(
                msg.uhura_port.as_ref(),
                msg.uhura_address.as_ref(),
                msg.uhura_socket.as_ref(),
            );
            let projector_listener = Listener::new(
                msg.projector_port.as_ref(),
                msg.projector_address.as_ref(),
                msg.projector_socket.as_ref(),
            );
            let p = Self::new(
                namespace,
                &pidfile,
                uhura_listener,
                projector_listener,
                Pid::from_raw(msg.pid),
            );
            debug!("Uhura spawned with pid: {}", p.handle.pid);
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
        match kill(self.handle.pid, SIGINT) {
            Ok(_) => debug!(
                "stopped uhura running for namespace {} with pid {}",
                self.handle.namespace, self.handle.pid
            ),
            Err(err) => warn!(
                "failed stop uhura running for namespace {} with pid {}: {}",
                self.handle.namespace, self.handle.pid, err
            ),
        };
    }
}
