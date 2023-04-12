use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::process::{Command, Stdio};
use std::sync::Arc;

use log::{debug, error, info, warn};
use nix::{
    fcntl::OFlag,
    unistd::{pipe2, Pid},
};
use serde::Deserialize;

use crate::config::HolodekkConfig;
use crate::core::entities::Projector;
use crate::utils::ConnectionInfo;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum SpawnError {
    #[error("Error launching projector: {0:?}")]
    BadExit(std::process::ExitStatus),
    #[error("Error synchronizing with projector process")]
    SyncError(String),
}

#[derive(Debug, Deserialize)]
struct MessageProjectorPidParent {
    pid: i32,
}

pub fn spawn_projector(
    config: Arc<HolodekkConfig>,
    namespace: &str,
) -> std::result::Result<Projector, SpawnError> {
    // Setup a pipe so we can be notified when the projector is fully up
    let (parent_fd, child_fd) = pipe2(OFlag::empty()).unwrap();
    let mut sync_pipe = unsafe { File::from_raw_fd(parent_fd) };

    let mut root_path = config.root_path.clone();
    root_path.push(namespace);

    // Ensure the root directory exists
    if !root_path.exists() {
        info!("Creating projector root directory: {}", root_path.display());
        fs::create_dir_all(&root_path).expect("Failed to create root directory for projector");
    }

    let mut pidfile = root_path.clone();
    pidfile.push("uhura.pid");

    let mut uhura_sock = root_path.clone();
    uhura_sock.push("uhura.sock");

    let mut projector_sock = root_path.clone();
    projector_sock.push("projector.sock");

    let mut uhura = config.bin_path.clone();
    uhura.push("uhura");

    let mut command = Command::new(uhura);
    command.arg("--projector-root");
    command.arg(&root_path);
    command.arg("--fleet");
    command.arg(&config.fleet);
    command.arg("--namespace");
    command.arg(namespace);
    command.arg("--pidfile");
    command.arg(&pidfile);
    command.arg("--sync-pipe");
    command.arg(child_fd.to_string());

    let uhura_listener = ConnectionInfo::unix(&uhura_sock);
    command.arg("--uhura-socket");
    command.arg(uhura_sock);
    let projector_listener = ConnectionInfo::unix(&projector_sock);
    command.arg("--projector-socket");
    command.arg(projector_sock);

    info!("Launching uhura with: {:?}", command);
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .status()
        .expect("Unable to spawn projector");
    info!("Status: {}", status);

    if status.success() {
        let mut buf = [0; 256];
        let bytes_read = sync_pipe
            .read(&mut buf)
            .expect("Unable to receive pid from spawned projector");
        let pid = match serde_json::from_slice::<MessageProjectorPidParent>(&buf[0..bytes_read]) {
            Ok(msg) => Pid::from_raw(msg.pid),
            Err(err) => {
                warn!("Failed to receive pid from projector: {}", err);
                // try to read it from the pidfile
                let contents =
                    fs::read_to_string(&pidfile).expect("Should have been able to read pid file");
                let pid: i32 = contents
                    .parse()
                    .expect("Unable to convert pidfile contents to pid");
                Pid::from_raw(pid)
            }
        };
        let p = Projector::new(
            config.fleet.as_str(),
            namespace,
            &pidfile,
            uhura_listener,
            projector_listener,
            pid,
        );
        debug!("Uhura spawned with pid: {}", p.pid);
        drop(sync_pipe);
        Ok(p)
    } else {
        error!("failed to launch uhura");
        Err(SpawnError::BadExit(status))
    }
}
