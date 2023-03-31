use std::fmt;
use std::fs::File;
use std::io::Read;
use std::net::Ipv4Addr;
use std::os::unix::io::FromRawFd;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use log::{debug, warn};

use nix::{
    fcntl::OFlag,
    sys::signal::{kill, SIGINT},
    unistd::{pipe2, Pid},
};

use serde::Deserialize;

use uuid::Uuid;

use crate::{Error, Result};

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
    pub admin_listener: Listener,
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
    admin_port: Option<u16>,
    admin_address: Option<Ipv4Addr>,
    admin_socket: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Projector {
    handle: ProjectorHandle,
}

impl Projector {
    pub fn new(
        namespace: &str,
        pidfile: &PathBuf,
        admin_listener: Listener,
        projector_listener: Listener,
        pid: Pid,
    ) -> Self {
        Self {
            handle: ProjectorHandle {
                id: Uuid::new_v4(),
                namespace: namespace.to_string(),
                pidfile: pidfile.to_owned(),
                admin_listener,
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

    pub fn pidfile(&self) -> &PathBuf {
        &self.handle.pidfile
    }

    pub fn pid(&self) -> &Pid {
        &self.handle.pid
    }

    pub fn admin_listener(&self) -> &Listener {
        &self.handle.admin_listener
    }

    pub fn projector_listener(&self) -> &Listener {
        &self.handle.projector_listener
    }

    pub fn stop(&self) -> Result<()> {
        match kill(self.handle.pid, SIGINT) {
            Ok(_) => {
                debug!(
                    "stopped uhura running for namespace {} with pid {}",
                    self.handle.namespace, self.handle.pid
                );
                Ok(())
            }
            Err(err) => {
                warn!(
                    "failed stop uhura running for namespace {} with pid {}: {}",
                    self.handle.namespace, self.handle.pid, err
                );
                Ok(())
            }
        }
    }

    pub fn spawn(
        namespace: &str,
        root: &PathBuf,
        admin_port: Option<u16>,
        projector_port: Option<u16>,
    ) -> Result<Projector> {
        // Setup a pipe so we can be notified when the projector is fully up
        let (parent_fd, child_fd) = pipe2(OFlag::empty()).unwrap();
        let mut sync_pipe = unsafe { File::from_raw_fd(parent_fd) };

        let mut pidfile = root.clone();
        pidfile.push("uhura.pid");

        let mut command = Command::new(
            "/home/jdubz/code/gitlab/holodekk/holodekk/target/debug/uhura".to_string(),
        );
        command.arg("--namespace".to_string());
        command.arg(namespace.to_string());
        command.arg("--pidfile".to_string());
        command.arg(pidfile.clone().into_os_string().into_string().unwrap());
        command.arg("--sync-pipe".to_string());
        command.arg(child_fd.to_string());

        if admin_port.is_some() {
            command.arg("--admin-port".to_string());
            command.arg(admin_port.unwrap().to_string());
        } else {
            let mut socket = root.clone();
            socket.push("admin.sock");
            command.arg("--admin-socket".to_string());
            command.arg(socket.clone().into_os_string().into_string().unwrap());
        }

        if projector_port.is_some() {
            command.arg("--projector-port".to_string());
            command.arg(projector_port.unwrap().to_string());
        } else {
            let mut socket = root.clone();
            socket.push("projector.sock");
            command.arg("--projector-socket".to_string());
            command.arg(socket.clone().into_os_string().into_string().unwrap());
        }

        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env_clear()
            .status()
            .expect("Failed to start uhura");

        if status.success() {
            let mut buf = [0; 256];
            let bytes_read = sync_pipe.read(&mut buf)?;
            let msg: MessageProjectorPidParent = serde_json::from_slice(&buf[0..bytes_read])?;
            let admin_listener = Listener::new(
                msg.admin_port.as_ref(),
                msg.admin_address.as_ref(),
                msg.admin_socket.as_ref(),
            );
            let projector_listener = Listener::new(
                msg.projector_port.as_ref(),
                msg.projector_address.as_ref(),
                msg.projector_socket.as_ref(),
            );
            let p = Self::new(
                namespace,
                &pidfile,
                admin_listener,
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
