use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use log::{debug, error, info, warn};
use nix::{
    fcntl::OFlag,
    sys::signal::{kill, SIGINT},
    unistd::{pipe2, Pid},
};
use serde::Deserialize;

use crate::config::{HolodekkConfig, ProjectorConfig};
use crate::core::subroutine_definitions::entities::SubroutineDefinition;
use crate::utils::Worker;

use super::entities::{Subroutine, SubroutineStatus};

#[derive(Debug)]
pub enum SubroutineCommand {
    Spawn {
        namespace: String,
        definition: SubroutineDefinition,
        resp: tokio::sync::oneshot::Sender<std::result::Result<Subroutine, SpawnError>>,
    },
    Shutdown {
        subroutine: Subroutine,
        definition: SubroutineDefinition,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), ShutdownError>>,
    },
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum SpawnError {
    #[error("Error launching subroutine: {0:?}")]
    BadExit(std::process::ExitStatus),
    #[error("Error synchronizing with subroutine process")]
    SyncError(String),
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ShutdownError {
    #[error("Failed to send SIGINT to subroutine process")]
    Kill(#[from] nix::Error),
}

#[derive(Debug, Deserialize)]
struct MessageSubroutinePidParent {
    pid: i32,
}

#[derive(Debug)]
pub struct SubroutinesWorker {
    task_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
    pub cmd_tx: Option<tokio::sync::mpsc::Sender<SubroutineCommand>>,
}

impl SubroutinesWorker {
    pub fn new(
        task_handle: tokio::task::JoinHandle<()>,
        cmd_tx: tokio::sync::mpsc::Sender<SubroutineCommand>,
    ) -> Self {
        Self {
            task_handle: RwLock::new(Some(task_handle)),
            cmd_tx: Some(cmd_tx),
        }
    }
}

#[async_trait]
impl Worker for SubroutinesWorker {
    type Command = SubroutineCommand;

    fn sender(&self) -> Option<tokio::sync::mpsc::Sender<SubroutineCommand>> {
        self.cmd_tx.as_ref().cloned()
    }

    async fn stop(&mut self) {
        if let Some(cmd_tx) = self.cmd_tx.take() {
            drop(cmd_tx);
        }
        let task_handle = self.task_handle.write().unwrap().take().unwrap();
        task_handle.await.unwrap()
    }
}

pub fn start_worker<C>(config: Arc<C>) -> SubroutinesWorker
where
    C: HolodekkConfig + ProjectorConfig,
{
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(32);
    let task_handle = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                SubroutineCommand::Spawn {
                    namespace,
                    definition,
                    resp,
                } => {
                    println!("spawning {}:{}", namespace, &definition.name());
                    let subroutine =
                        spawn_subroutine(config.clone(), &namespace, definition.clone()).unwrap();
                    resp.send(Ok(subroutine)).unwrap();
                }
                SubroutineCommand::Shutdown {
                    subroutine,
                    definition,
                    resp,
                } => {
                    shutdown_subroutine(subroutine.clone(), definition.clone()).unwrap();
                    resp.send(Ok(())).unwrap();
                }
            }
        }
    });
    SubroutinesWorker::new(task_handle, cmd_tx)
}

pub async fn subroutine_manager<C>(
    config: Arc<C>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<SubroutineCommand>,
) where
    C: HolodekkConfig + ProjectorConfig,
{
    let config = config.clone();
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            SubroutineCommand::Spawn {
                namespace,
                definition,
                resp,
            } => {
                println!("spawning {}:{}", namespace, &definition.name());
                let subroutine =
                    spawn_subroutine(config.clone(), &namespace, definition.clone()).unwrap();
                resp.send(Ok(subroutine)).unwrap();
            }
            SubroutineCommand::Shutdown {
                subroutine,
                definition,
                resp,
            } => {
                shutdown_subroutine(subroutine.clone(), definition.clone()).unwrap();
                resp.send(Ok(())).unwrap();
            }
        }
    }
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
    root_path.push(definition.name());

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
    command.arg(definition.name());
    command.arg("--pidfile");
    command.arg(&shim_pidfile);
    command.arg("--subroutine-directory");
    command.arg(definition.path());
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
        let mut i = Subroutine::new(config.fleet(), namespace, subroutine_root, definition.id());
        i.status = SubroutineStatus::Running(pid.as_raw() as u32);
        debug!("Subroutine spawned with pid: {}", pid);
        drop(sync_pipe);
        Ok(i)
    } else {
        error!("failed to launch uhura");
        Err(SpawnError::BadExit(status))
    }
}

pub fn shutdown_subroutine(
    subroutine: Subroutine,
    definition: SubroutineDefinition,
) -> std::result::Result<(), ShutdownError> {
    // TODO: check to see if the subroutine is still running before blindly killing it
    match subroutine.status {
        SubroutineStatus::Running(pid) => match kill(Pid::from_raw(pid as i32), SIGINT) {
            Ok(_) => {
                debug!(
                    "stopped subroutine {} running in namespace {} with pid {}",
                    definition.name(),
                    subroutine.namespace,
                    pid
                );
                Ok(())
            }
            Err(err) => {
                warn!(
                    "failed stop subroutine {} running in namespace {} with pid {}: {}",
                    definition.name(),
                    subroutine.namespace,
                    pid,
                    err
                );
                Err(ShutdownError::from(err))
            }
        },
        status => {
            warn!(
                "Requested shutdown for subroutine that is not running: {:?}",
                status
            );
            Ok(())
        }
    }
}
