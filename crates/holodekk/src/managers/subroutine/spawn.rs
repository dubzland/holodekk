use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use log::{debug, error, info, warn};
use nix::{
    fcntl::OFlag,
    unistd::{pipe2, Pid},
};
use serde::Deserialize;

use crate::config::{HolodekkConfig, ProjectorConfig};
use crate::core::entities::{Subroutine, SubroutineDefinition, SubroutineStatus};
use crate::core::repositories::RepositoryId;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum SpawnError {
    #[error("Error launching subroutine: {0:?}")]
    BadExit(std::process::ExitStatus),
    #[error("Error synchronizing with subroutine process")]
    SyncError(String),
}

#[derive(Debug, Deserialize)]
struct MessageSubroutinePidParent {
    pid: i32,
}

fn build_command<C>(
    config: Arc<C>,
    definition: SubroutineDefinition,
    child_fd: i32,
) -> (PathBuf, PathBuf, Command)
where
    C: HolodekkConfig + ProjectorConfig,
{
    let mut root_path = config.projector_path().clone();
    root_path.push(definition.name.clone());

    // Ensure the root directory exists
    if !root_path.exists() {
        info!("Creating projector root directory: {}", root_path.display());
        fs::create_dir_all(&root_path).expect("Failed to create root directory for projector");
    }

    let mut shim_pidfile = root_path.clone();
    shim_pidfile.push("shim.pid");

    let mut subroutine_pidfile = root_path.clone();
    subroutine_pidfile.push("subroutine.pid");

    let mut log_file = root_path.clone();
    log_file.push("subroutine.log");

    let mut log_socket = root_path.clone();
    log_socket.push("log.sock");

    let mut uhura_sock = root_path.clone();
    uhura_sock.push("uhura.sock");

    let mut projector_sock = root_path.clone();
    projector_sock.push("projector.sock");

    let mut subroutine_bin = config.paths().bin().clone();
    subroutine_bin.push("holodekk-subroutine");

    let mut command = Command::new(subroutine_bin);
    command.arg("--name");
    command.arg(definition.name.clone());
    command.arg("--pidfile");
    command.arg(&shim_pidfile);
    command.arg("--subroutine-directory");
    command.arg(&definition.path);
    command.arg("--subroutine-pidfile");
    command.arg(&subroutine_pidfile);
    command.arg("--log-file");
    command.arg(log_file);
    command.arg("--subroutine");
    command.arg("default");
    // TODO: Finish connecting subroutine to projector
    command.arg("--projector-port");
    command.arg("1234");
    command.arg("--log-socket");
    command.arg(log_socket);
    command.arg("--sync-pipe");
    command.arg(child_fd.to_string());

    (subroutine_pidfile, root_path, command)
}

pub fn spawn_subroutine<C>(
    config: Arc<C>,
    namespace: &str,
    definition: SubroutineDefinition,
) -> std::result::Result<Subroutine, SpawnError>
where
    C: HolodekkConfig + ProjectorConfig,
{
    // Setup a pipe so we can be notified when the projector is fully up
    let (parent_fd, child_fd) = pipe2(OFlag::empty()).unwrap();
    let mut sync_pipe = unsafe { File::from_raw_fd(parent_fd) };

    let (pidfile, subroutine_root, mut command) =
        build_command(config.clone(), definition.clone(), child_fd);

    info!("Launching subroutine with: {:?}", command);
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .status()
        .expect("Unable to spawn subroutine");
    info!("Status: {}", status);

    if status.success() {
        let mut buf = [0; 256];
        let bytes_read = sync_pipe
            .read(&mut buf)
            .expect("Unable to receive pid from spawned subroutine");
        let pid = match serde_json::from_slice::<MessageSubroutinePidParent>(&buf[0..bytes_read]) {
            Ok(msg) => Pid::from_raw(msg.pid),
            Err(err) => {
                warn!("Failed to receive pid from subroutine: {}", err);
                // try to read it from the pidfile
                let contents =
                    fs::read_to_string(pidfile).expect("Should have been able to read pid file");
                let pid: i32 = contents
                    .parse()
                    .expect("Unable to convert pidfile contents to pid");
                Pid::from_raw(pid)
            }
        };
        let mut i = Subroutine::new(config.fleet(), namespace, subroutine_root, &definition.id());
        i.status = SubroutineStatus::Running(pid.as_raw() as u32);
        debug!("Subroutine spawned with pid: {}", pid);
        drop(sync_pipe);
        Ok(i)
    } else {
        error!("failed to launch uhura");
        Err(SpawnError::BadExit(status))
    }
}
