use std::sync::Arc;

use anyhow::Context;
use log::{debug, info, warn};
use nix::{sys::signal::kill, unistd::Pid};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::{
    config::HolodekkConfig,
    core::projectors::{
        entities::{ProjectorEntity, ProjectorStatus},
        repositories::{ProjectorsQuery, ProjectorsRepository},
        worker::{
            ProjectorsRequest, ProjectorsWorker, SpawnError as ProjectorSpawnError,
            TerminationError as ProjectorTerminationError,
        },
    },
    core::subroutine_definitions::entities::SubroutineDefinitionEntity,
    core::subroutines::{
        entities::SubroutineEntity,
        worker::{
            SpawnError as SubroutineSpawnError, SubroutinesRequest, SubroutinesWorker,
            TerminationError as SubroutineTerminationError,
        },
    },
};

#[derive(thiserror::Error, Debug)]
pub enum DirectorError {
    #[error("Failed to spawn projector")]
    ProjectorSpawn(#[from] ProjectorSpawnError),
    #[error("Failed to terminate projector")]
    ProjectorTermination(#[from] ProjectorTerminationError),
    #[error("Failed to spawn subroutine")]
    SubroutineSpawn(#[from] SubroutineSpawnError),
    #[error("Failed to terminate subroutine")]
    SubroutineTermination(#[from] SubroutineTerminationError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Debug)]
pub enum DirectorRequest {
    Shutdown {
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), DirectorError>>,
    },
    SpawnProjector {
        namespace: String,
        resp: tokio::sync::oneshot::Sender<std::result::Result<ProjectorEntity, DirectorError>>,
    },
    TerminateProjector {
        projector: ProjectorEntity,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), DirectorError>>,
    },
    SpawnSubroutine {
        projector: ProjectorEntity,
        definition: SubroutineDefinitionEntity,
        resp: tokio::sync::oneshot::Sender<std::result::Result<SubroutineEntity, DirectorError>>,
    },
    TerminateSubroutine {
        subroutine: SubroutineEntity,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), DirectorError>>,
    },
}

pub struct DirectorServer<C, R>
where
    R: ProjectorsRepository,
    C: HolodekkConfig,
{
    config: Arc<C>,
    repo: Arc<R>,
    director_receiver: Receiver<DirectorRequest>,
    director_sender: Option<Sender<DirectorRequest>>,
    projectors_sender: Option<Sender<ProjectorsRequest>>,
    projectors_handle: Option<JoinHandle<()>>,
    subroutines_sender: Option<Sender<SubroutinesRequest>>,
    subroutines_handle: Option<JoinHandle<()>>,
}

impl<C, R> DirectorServer<C, R>
where
    R: ProjectorsRepository + 'static,
    C: HolodekkConfig,
{
    pub fn start(config: Arc<C>, repo: Arc<R>) -> (JoinHandle<()>, Sender<DirectorRequest>) {
        let (projectors_sender, projectors_receiver) = channel(32);
        let (subroutines_sender, subroutines_receiver) = channel(32);
        let (director_sender, director_receiver) = channel(32);

        let projectors_handle = ProjectorsWorker::start(config.clone(), projectors_receiver);
        let subroutines_handle = SubroutinesWorker::start(config.clone(), subroutines_receiver);

        let return_sender = director_sender.clone();

        let task_handle = tokio::spawn(async move {
            let mut director = DirectorServer {
                config,
                repo,
                director_sender: Some(director_sender),
                director_receiver,
                projectors_sender: Some(projectors_sender),
                projectors_handle: Some(projectors_handle),
                subroutines_sender: Some(subroutines_sender),
                subroutines_handle: Some(subroutines_handle),
            };

            director.run().await;
        });

        (task_handle, return_sender)
    }

    async fn run(&mut self) {
        self.check_projectors().await.unwrap();
        loop {
            tokio::select! {
                Some(request) = self.director_receiver.recv() => {
                    self.process_request(request).await
                }
                else => {
                    debug!("All senders closed.  Exiting.");
                    break;
                }
            }
        }
    }

    async fn process_request(&mut self, request: DirectorRequest) {
        match request {
            DirectorRequest::Shutdown { resp } => {
                info!("Director received shutdown.");
                let response = self.process_shutdown().await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to Shutdown request (receiver dropped)");
                }
            }
            DirectorRequest::SpawnProjector { namespace, resp } => {
                let response = self.process_projector_spawn(namespace).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to SpawnProjector request (receiver dropped)");
                }
            }
            DirectorRequest::TerminateProjector { projector, resp } => {
                let response = self.process_projector_terminate(projector).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to TerminateProjector request (receiver dropped)");
                }
            }
            DirectorRequest::SpawnSubroutine {
                projector,
                definition,
                resp,
            } => {
                let response = self.process_subroutine_spawn(projector, definition).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to SpawnSubroutine request (receiver dropped)");
                }
            }
            DirectorRequest::TerminateSubroutine { subroutine, resp } => {
                let response = self.process_subroutine_terminate(subroutine).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to TerminateSubroutine request (receiver dropped)");
                }
            }
        }
    }

    async fn process_shutdown(&mut self) -> std::result::Result<(), DirectorError> {
        info!("Shutting down Projectors worker ...");
        let projectors_sender = self.projectors_sender.take().unwrap();
        drop(projectors_sender);

        info!("Shutting down Subroutines worker ...");
        let subroutines_sender = self.subroutines_sender.take().unwrap();
        drop(subroutines_sender);

        let projectors_handle = self.projectors_handle.take().unwrap();
        if let Err(err) = projectors_handle.await {
            warn!("Projectors worker exited with non-success: {}", err);
        }

        let subroutines_handle = self.subroutines_handle.take().unwrap();
        if let Err(err) = subroutines_handle.await {
            warn!("Subroutines worker exited with non-success: {}", err);
        }

        let director_sender = self.director_sender.take().unwrap();
        drop(director_sender);

        Ok(())
    }

    async fn process_projector_spawn(
        &self,
        namespace: String,
    ) -> std::result::Result<ProjectorEntity, DirectorError> {
        let (worker_tx, worker_rx) = tokio::sync::oneshot::channel();
        self.projectors_sender
            .as_ref()
            .cloned()
            .unwrap()
            .send(ProjectorsRequest::Spawn {
                namespace,
                resp: worker_tx,
            })
            .await
            .context("Failed to send spawn request to worker")?;
        let projector = worker_rx
            .await
            .context("Failed to receive response from Projectors worker for spawn request")??;
        Ok(projector)
    }

    async fn process_projector_terminate(
        &self,
        projector: ProjectorEntity,
    ) -> std::result::Result<(), DirectorError> {
        let (worker_tx, worker_rx) = tokio::sync::oneshot::channel();
        self.projectors_sender
            .as_ref()
            .cloned()
            .unwrap()
            .send(ProjectorsRequest::Terminate {
                projector,
                resp: worker_tx,
            })
            .await
            .context("failed to send terminate request to Projectors worker")?;
        worker_rx
            .await
            .context("Failed to receive response from Projectors worker for terminate request")??;
        Ok(())
    }

    async fn process_subroutine_spawn(
        &self,
        projector: ProjectorEntity,
        definition: SubroutineDefinitionEntity,
    ) -> std::result::Result<SubroutineEntity, DirectorError> {
        let (worker_tx, worker_rx) = tokio::sync::oneshot::channel();
        self.subroutines_sender
            .as_ref()
            .cloned()
            .unwrap()
            .send(SubroutinesRequest::Spawn {
                projector,
                definition,
                resp: worker_tx,
            })
            .await
            .context("Failed to send spawn request to worker")?;
        let projector = worker_rx
            .await
            .context("Failed to receive response from Subroutines worker for spawn request")??;
        Ok(projector)
    }

    async fn process_subroutine_terminate(
        &self,
        subroutine: SubroutineEntity,
    ) -> std::result::Result<(), DirectorError> {
        let (worker_tx, worker_rx) = tokio::sync::oneshot::channel();
        self.subroutines_sender
            .as_ref()
            .cloned()
            .unwrap()
            .send(SubroutinesRequest::Terminate {
                subroutine,
                resp: worker_tx,
            })
            .await
            .context("failed to send terminate request to Subroutines worker")?;
        worker_rx.await.context(
            "Failed to receive response from Subroutines worker for terminate request",
        )??;
        Ok(())
    }

    pub async fn check_projectors(&self) -> Result<(), DirectorError> {
        // get the list of running projectors from repository
        let mut repo_projectors = self
            .repo
            .projectors_find(ProjectorsQuery::default())
            .await
            .unwrap();

        // get the list of actually running projectors
        let mut running_projectors: Vec<ProjectorEntity> = std::fs::read_dir(self.config.projectors_root())
            .unwrap()
            .filter_map(|e| {
                let entry = e.unwrap();
                let mut uhura_pidfile = entry.path();
                uhura_pidfile.push("uhura.pid");
                if uhura_pidfile.try_exists().unwrap() {
                    let pid = std::fs::read_to_string(&uhura_pidfile)
                        .expect("Should have been able to read pid file");
                    let pid: i32 = pid
                        .parse()
                        .expect("Unable to convert pidfile contents to pid");
                    match kill(Pid::from_raw(pid), None) {
                        Err(_) => {
                            info!(
                                "Found existing pidfile at {}, but no process found. Removing directory",
                                uhura_pidfile.display()
                            );
                            warn!("Removing directory: {}", entry.path().display());
                            std::fs::remove_dir_all(entry.path()).unwrap();
                            None
                        }
                        Ok(_) => {
                            let namespace = entry.path();
                            let namespace = namespace.iter().last().unwrap().to_str().unwrap();

                            let mut p = ProjectorEntity::new(
                                namespace,
                                entry.path(),
                            );
                            p.set_status(ProjectorStatus::Running(pid as u32));
                            Some(p)
                        }
                    }
                } else {
                    None
                }
            })
            .collect();

        // synchronize
        while let Some(running) = running_projectors.pop() {
            if let Some(projector) = repo_projectors
                .iter()
                .position(|p| p.status() == running.status())
            {
                info!(
                    "Found dead projector: {:?} ... removing from repo",
                    projector
                );
                repo_projectors.remove(projector);
            } else {
                info!("Found orphan projector: {:?} ... cleaning up", running);
                let projectors_sender = self.projectors_sender.as_ref().cloned().unwrap();
                let (worker_tx, worker_rx) = tokio::sync::oneshot::channel();
                projectors_sender
                    .send(ProjectorsRequest::Terminate {
                        projector: running,
                        resp: worker_tx,
                    })
                    .await
                    .unwrap();
                worker_rx.await.unwrap().unwrap();
            }
        }

        // at this point, anything still in repo_projectors isn't running.  trash it.
        for projector in repo_projectors {
            debug!("cleaning up dead projector: {}", projector.id());
            self.repo.projectors_delete(projector.id()).await.unwrap();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::prelude::*;
    use std::sync::Arc;

    use tempfile::tempdir;

    use crate::config::fixtures::MockConfig;
    use crate::repositories::memory::{MemoryDatabase, MemoryRepository};

    use super::*;

    fn fork_projector() -> std::result::Result<nix::unistd::Pid, nix::Error> {
        let child = match unsafe { nix::unistd::fork() } {
            Ok(nix::unistd::ForkResult::Parent { child }) => child,
            Ok(nix::unistd::ForkResult::Child) => {
                // Redirect all streams to /dev/null
                let dev_null_rd = nix::fcntl::open(
                    "/dev/null",
                    nix::fcntl::OFlag::O_RDONLY | nix::fcntl::OFlag::O_CLOEXEC,
                    nix::sys::stat::Mode::empty(),
                )
                .expect("Opening /dev/null for reading failed");
                let dev_null_wr = nix::fcntl::open(
                    "/dev/null",
                    nix::fcntl::OFlag::O_WRONLY | nix::fcntl::OFlag::O_CLOEXEC,
                    nix::sys::stat::Mode::empty(),
                )
                .expect("Opening /dev/null for reading failed");
                // let (dev_null_rd, dev_null_wr) = open_dev_null();
                nix::unistd::dup2(dev_null_rd, crate::utils::libsee::STDIN_FILENO)
                    .expect("Failed to redirect stdin to /dev/null");
                nix::unistd::dup2(dev_null_wr, crate::utils::libsee::STDOUT_FILENO)
                    .expect("Failed to redirect stdout to /dev/null");
                nix::unistd::dup2(dev_null_wr, crate::utils::libsee::STDERR_FILENO)
                    .expect("Failed to redirect stderr to /dev/null");
                std::thread::sleep(std::time::Duration::from_secs(5));
                Pid::from_raw(-1)
            }
            Err(err) => {
                panic!("Error forking process: {}", err);
            }
        };
        Ok(child)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_projectors_removes_orphan_projectors() -> std::io::Result<()> {
        // setup temp directories
        let temp = tempdir().unwrap();
        let root_path = temp.into_path();
        let mut data_root = root_path.clone();
        data_root.push("data");
        let mut exec_root = root_path.clone();
        exec_root.push("exec");

        let config = Arc::new(MockConfig::new(data_root, exec_root));

        // setup a fake projector
        let pid = fork_projector()?;
        let mut root = config.projectors_root().to_owned();
        root.push("test");
        let mut projector = ProjectorEntity::new("test", &root);
        projector.set_status(ProjectorStatus::Running(pid.as_raw() as u32));
        fs::create_dir_all(projector.root())?;
        let mut file = File::create(projector.pidfile())?;
        file.write_all(format!("{}", pid.as_raw()).as_bytes())?;

        let db = Arc::new(MemoryDatabase::new());
        let repo = Arc::new(MemoryRepository::new(db.clone()));

        let (director_handle, director_sender) = DirectorServer::start(config, repo);
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        director_sender
            .send(DirectorRequest::Shutdown { resp: resp_tx })
            .await
            .unwrap();
        drop(director_sender);
        resp_rx.await.unwrap().unwrap();
        director_handle.await.unwrap();

        assert!(!projector.root().exists());

        Ok(())
    }
}
