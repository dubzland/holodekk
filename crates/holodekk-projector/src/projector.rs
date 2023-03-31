use std::fmt;
use std::fs::File;
use std::io::Read;
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
pub struct ProjectorHandle {
    pub id: Uuid,
    pub namespace: String,
    pub pidfile: PathBuf,
    pub pid: Pid,
    pub port: u16,
    pub rpc_port: u16,
}

impl fmt::Display for ProjectorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, Debug)]
pub struct Projector {
    handle: ProjectorHandle,
}

#[derive(Debug, Deserialize)]
struct MessageProjectorPidParent {
    pid: i32,
    port: u16,
    rpc_port: u16,
}

impl Projector {
    pub fn new(namespace: &str, pidfile: &PathBuf, port: u16, rpc_port: u16, pid: Pid) -> Self {
        Self {
            handle: ProjectorHandle {
                id: Uuid::new_v4(),
                namespace: namespace.to_string(),
                pidfile: pidfile.to_owned(),
                port,
                rpc_port,
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

    pub fn port(&self) -> u16 {
        self.handle.port
    }

    pub fn rpc_port(&self) -> u16 {
        self.handle.rpc_port
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
        port: Option<u16>,
        rpc_port: Option<u16>,
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

        if port.is_some() {
            command.arg("--port".to_string());
            command.arg(port.unwrap().to_string());
        }

        if rpc_port.is_some() {
            command.arg("--rpc-port".to_string());
            command.arg(rpc_port.unwrap().to_string());
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
            // match close(child_fd) {
            //     Ok(_) => {}
            //     Err(err) => {
            //         warn!("Failed to close child end of sync pipe: {}", err);
            //     }
            // };
            let msg: MessageProjectorPidParent = serde_json::from_slice(&buf[0..bytes_read])?;
            let p = Self::new(
                namespace,
                &pidfile,
                msg.port,
                msg.rpc_port,
                Pid::from_raw(msg.pid),
            );
            debug!("Uhura spawned with pid: {}", p.handle.pid);
            drop(sync_pipe);
            println!(
                "Uhura spawned on ports {}/{} with pid {}",
                p.handle.port, p.handle.rpc_port, p.handle.pid
            );
            Ok(p)
        } else {
            warn!("failed to launch uhura");
            Err(Error::LaunchError(status))
        }
    }
}
