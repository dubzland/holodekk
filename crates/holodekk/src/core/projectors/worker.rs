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

use crate::config::HolodekkConfig;
use crate::utils::{ConnectionInfo, TaskHandle, Worker};

use super::entities::Projector;

#[derive(Debug)]
pub enum ProjectorCommand {
    Spawn {
        namespace: String,
        resp: tokio::sync::oneshot::Sender<std::result::Result<Projector, SpawnError>>,
    },
    Shutdown {
        projector: Projector,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), ShutdownError>>,
    },
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum SpawnError {
    #[error("Error launching projector: {0:?}")]
    BadExit(std::process::ExitStatus),
    #[error("Error synchronizing with projector process")]
    SyncError(String),
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ShutdownError {
    #[error("Failed to send SIGINT to projector process")]
    Kill(#[from] nix::Error),
}

#[derive(Debug, Deserialize)]
struct MessageProjectorPidParent {
    pid: i32,
}

#[derive(Debug)]
pub struct ProjectorsWorker {
    task_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
    pub cmd_tx: Option<tokio::sync::mpsc::Sender<ProjectorCommand>>,
}

impl ProjectorsWorker {
    pub fn new(
        task_handle: tokio::task::JoinHandle<()>,
        cmd_tx: tokio::sync::mpsc::Sender<ProjectorCommand>,
    ) -> Self {
        Self {
            task_handle: RwLock::new(Some(task_handle)),
            cmd_tx: Some(cmd_tx),
        }
    }
}

#[async_trait]
impl TaskHandle for ProjectorsWorker {
    async fn stop(&mut self) {
        if let Some(cmd_tx) = self.cmd_tx.take() {
            drop(cmd_tx);
        }
        let task_handle = self.task_handle.write().unwrap().take().unwrap();
        task_handle.await.unwrap()
    }
}

impl Worker for ProjectorsWorker {
    type Command = ProjectorCommand;

    fn sender(&self) -> Option<tokio::sync::mpsc::Sender<ProjectorCommand>> {
        self.cmd_tx.as_ref().cloned()
    }
}

pub fn start_worker<C>(config: Arc<C>) -> ProjectorsWorker
where
    C: HolodekkConfig,
{
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(32);
    let task_handle = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                ProjectorCommand::Spawn { namespace, resp } => {
                    println!("spawning {}", namespace);
                    let projector = spawn_projector(config.clone(), &namespace).unwrap();
                    resp.send(Ok(projector)).unwrap();
                }
                ProjectorCommand::Shutdown { projector, resp } => {
                    shutdown_projector(projector.clone()).unwrap();
                    resp.send(Ok(())).unwrap();
                }
            }
        }
    });
    ProjectorsWorker::new(task_handle, cmd_tx)
}

fn build_command<C>(
    config: Arc<C>,
    namespace: &str,
    child_fd: i32,
) -> (PathBuf, ConnectionInfo, ConnectionInfo, Command)
where
    C: HolodekkConfig,
{
    let mut root_path = config.paths().projectors().clone();
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

    let mut uhura = config.paths().bin().clone();
    uhura.push("uhura");

    let mut command = Command::new(uhura);
    command.arg("--projector-root");
    command.arg(&root_path);
    command.arg("--holodekk-bin");
    command.arg(config.paths().bin());
    command.arg("--fleet");
    command.arg(config.fleet());
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

    (pidfile, uhura_listener, projector_listener, command)
}

pub fn spawn_projector<C>(
    config: Arc<C>,
    namespace: &str,
) -> std::result::Result<Projector, SpawnError>
where
    C: HolodekkConfig,
{
    // Setup a pipe so we can be notified when the projector is fully up
    let (parent_fd, child_fd) = pipe2(OFlag::empty()).unwrap();
    let mut sync_pipe = unsafe { File::from_raw_fd(parent_fd) };

    let (pidfile, uhura_listener, projector_listener, mut command) =
        build_command(config.clone(), namespace, child_fd);

    debug!("Launching uhura with: {:?}", command);
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .status()
        .expect("Unable to spawn projector");
    debug!("Status: {}", status);

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
            config.fleet(),
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

pub fn shutdown_projector(projector: Projector) -> std::result::Result<(), ShutdownError> {
    // TODO: check to see if uhura is still running before blindly killing it
    match kill(projector.pid, SIGINT) {
        Ok(_) => {
            debug!(
                "stopped uhura running for namespace {} with pid {}",
                projector.namespace, projector.pid
            );
            Ok(())
        }
        Err(err) => {
            warn!(
                "failed stop uhura running for namespace {} with pid {}: {}",
                projector.namespace, projector.pid, err
            );
            Err(ShutdownError::from(err))
        }
    }
}
