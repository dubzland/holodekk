use std::process::Command;
use std::sync::Arc;

use log::{debug, info, trace, warn};
use nix::{sys::signal::kill, unistd::Pid};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use holodekk::core::entities::{SceneEntity, SceneEntityId, SceneName};
use holodekk::core::ScenePaths;
use holodekk::enums::SceneStatus;
use holodekk::utils::{
    fs::ensure_directory,
    process::{daemonize, terminate_daemon, DaemonTerminationError, DaemonizeError},
};

use crate::config::HolodekkdConfig;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum SceneMessage {
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum SceneEvent {
    Started(i32),
}

#[derive(thiserror::Error, Debug)]
pub enum SceneError {
    #[error("Scene is invalid")]
    InvalidScene,
    #[error("Failed to launch projector process")]
    Daemonize(#[from] DaemonizeError),
    #[error("Failed to terminate projector process")]
    Termination(#[from] DaemonTerminationError),
    #[error("Error during cleanup")]
    Io(#[from] std::io::Error),
}

pub struct SceneHandle {
    pub sender: Option<Sender<SceneMessage>>,
    pub projector_events: Receiver<SceneEvent>,
    pub handle: JoinHandle<()>,
}

impl SceneHandle {
    pub async fn stop(mut self) -> Result<(), SceneError> {
        let sender = self.sender.take();
        if let Some(sender) = sender {
            sender.send(SceneMessage::Shutdown).await.unwrap();
            drop(sender);
        }
        debug!("Shutdown message sent.  Awiting Scene termination ...");
        if let Err(err) = self.handle.await {
            warn!("Error waiting for Scene termination: {}", err);
        }
        debug!("Scene termination complete.");
        Ok(())
    }
}

pub struct Scene {
    pub id: SceneEntityId,
    pub name: SceneName,
    pub status: SceneStatus,
    pub paths: ScenePaths,
    pub receiver: Receiver<SceneMessage>,
    pub event_sender: Sender<SceneEvent>,
    pub config: Arc<HolodekkdConfig>,
}

impl Scene {
    pub async fn start(
        config: Arc<HolodekkdConfig>,
        entity: &SceneEntity,
    ) -> std::result::Result<SceneHandle, SceneError> {
        let (messages_tx, messages_rx) = channel(32);
        let (events_tx, events_rx) = channel(32);
        let scene_id = entity.id.clone();
        let scene_name = entity.name.clone();
        let paths = ScenePaths::build(config.paths(), &scene_name);

        let handle = tokio::spawn(async move {
            let mut scene = Scene {
                id: scene_id,
                name: scene_name,
                status: SceneStatus::Unknown,
                paths,
                receiver: messages_rx,
                event_sender: events_tx,
                config,
            };

            scene.run().await;
        });

        Ok(SceneHandle {
            sender: Some(messages_tx),
            projector_events: events_rx,
            handle,
        })
    }

    async fn run(&mut self) {
        // check for projector process, and start if not running
        self.check_projector().await.unwrap();
        // monitor events
        loop {
            tokio::select! {
                Some(message) = self.receiver.recv() => {
                    info!("Received message: {:?}", message);
                    match message {
                        SceneMessage::Shutdown => {
                            debug!("Shutting down projector for scene {} ...", self.name);
                            self.stop_projector().await.unwrap();
                            debug!("Projector shutdown complete.");
                        }
                    }
                }
                else => {
                    debug!("All senders closed.  Exiting.");
                    break;
                }
            }
        }
    }

    async fn check_projector(&mut self) -> std::result::Result<(), SceneError> {
        let status = if self.paths.pidfile().try_exists().unwrap() {
            let pid = std::fs::read_to_string(self.paths.pidfile())
                .expect("Should have been able to read pid file");
            let pid: i32 = pid
                .parse()
                .expect("Unable to convert pidfile contents to pid");
            match kill(Pid::from_raw(pid), None) {
                Err(_) => {
                    info!(
                        "Found existing pidfile at {}, but no process found",
                        self.paths.pidfile().display()
                    );
                    SceneStatus::Crashed
                }
                Ok(_) => SceneStatus::Running(pid),
            }
        } else {
            SceneStatus::Unknown
        };

        if matches!(status, SceneStatus::Running(..)) {
            self.status = status;
            Ok(())
        } else {
            // start it up
            if let SceneStatus::Running(pid) = self
                .start_projector(&self.id, &self.name, &self.paths)
                .await?
            {
                self.event_sender
                    .send(SceneEvent::Started(pid))
                    .await
                    .unwrap();
                self.status = SceneStatus::Running(pid);
            }
            Ok(())
        }
    }

    async fn start_projector(
        &self,
        id: &SceneEntityId,
        name: &SceneName,
        paths: &ScenePaths,
    ) -> std::result::Result<SceneStatus, SceneError> {
        trace!("Scene::start_projector({:?}, {:?}, {:?}", id, name, paths);

        // ensure the root directory exists
        ensure_directory(paths.root()).unwrap();

        // build and execute the actual projector command
        let mut uhura = self.config.paths().bin_root().clone();
        uhura.push("uhura");

        let mut command = Command::new(uhura);
        command.arg("--id");
        command.arg(id);
        command.arg("--name");
        command.arg(name);

        let pid = daemonize(self.config.paths(), command, paths.pidfile())?;
        Ok(SceneStatus::Running(pid))
    }

    async fn stop_projector(&self) -> std::result::Result<(), SceneError> {
        trace!("Scene::stop_projector()");
        if let SceneStatus::Running(pid) = self.status {
            terminate_daemon(pid)?;
            std::fs::remove_dir_all(self.paths.root())?;
            debug!("Scene cleanup complete.");
        }

        Ok(())
    }
}

// pub async fn initialize_subroutines<C, R>(config: Arc<C>, repo: Arc<R>) -> super::Result<()>
// where
//     C: HolodekkConfig,
//     R: SubroutinesRepository + 'static,
// {
//     // get the list of running subroutines from repository
//     let mut repo_subroutines = repo.subroutines_find(SubroutinesQuery::default()).await;

//     // get the list of actually running subroutines
//     let mut running_subroutines: Vec<Subroutine> = std::fs::read_dir(config.subroutines_root())
//         .unwrap()
//         .filter_map(|e| {
//             let entry = e.unwrap();
//             let mut projector_dir = entry.path();
//             projector_dir.push("subroutines");
//             let namespace = entry.path();
//             let namespace = namespace.iter().last().unwrap().to_str().unwrap();
//             let subroutines = subroutines_for_projector(namespace, projector_dir).unwrap();
//             if subroutines.is_empty() {
//                 None
//             } else {
//                 Some(subroutines)
//             }
//         })
//         .flatten()
//         .collect();

//     // synchronize
//     while let Some(running) = running_subroutines.pop() {
//         if let Some(subroutine) = repo_subroutines
//             .iter()
//             .position(|s| s.status() == running.status())
//         {
//             info!(
//                 "Found dead subroutine: {:?} ... removing from repo",
//                 subroutine
//             );
//             repo_subroutines.remove(subroutine);
//         } else {
//             info!("Found missing subroutine: {:?} ... adding to repo", running);
//             repo.subroutines_create(running).await.unwrap();
//         }
//     }

//     // at this point, anything still in repo_projectors isn't running.  trash it.
//     for subroutine in repo_subroutines {
//         repo.subroutines_delete(&subroutine.id()).await.unwrap();
//     }

//     Ok(())
// }

// fn subroutines_for_projector<P: AsRef<Path>>(
//     namespace: &str,
//     path: P,
// ) -> super::Result<Vec<Subroutine>> {
//     if !path.as_ref().exists() {
//         return Ok(vec![]);
//     }
//     let subroutines = std::fs::read_dir(path)
//         .unwrap()
//         .filter_map(|e| {
//             let entry = e.unwrap();
//             let mut subroutine_pidfile = entry.path();
//             subroutine_pidfile.push("subroutine.pid");
//             if subroutine_pidfile.try_exists().unwrap() {
//                 let pid = std::fs::read_to_string(&subroutine_pidfile)
//                     .expect("Should have been able to read pid file");
//                 let pid: u32 = pid
//                     .parse()
//                     .expect("Unable to convert pidfile contents to pid");
//                 match kill(Pid::from_raw(pid as i32), None) {
//                     Err(_) => {
//                         info!(
//                             "Found existing pidfile at {}, but no process found. Removing directory",
//                             subroutine_pidfile.display()
//                         );
//                         warn!("Removing directory: {}", entry.path().display());
//                         std::fs::remove_dir_all(entry.path()).unwrap();
//                         None
//                     }
//                     Ok(_) => {
//                         let subroutine_path = entry.path();
//                         let subroutine_definition_id = subroutine_path.iter().last().unwrap().to_str().unwrap().to_string();
//                         let mut s = Subroutine::new(
//                             "".to_string(),
//                             namespace.to_string(),
//                             subroutine_path,
//                             subroutine_definition_id,
//                         );
//                         s.set_status(SubroutineStatus::Running(pid));
//                         Some(s)
//                     }
//                 }
//             } else {
//                 None
//             }
//         })
//         .collect();
//     Ok(subroutines)
// }

// #[cfg(test)]
// mod tests {
//     use std::fs::{self, File};
//     use std::io::prelude::*;
//     use std::sync::Arc;

//     use crate::core::repositories::memory::{MemoryDatabase, MemoryRepository};
//     use tempfile::tempdir;

//     use crate::config::fixtures::MockConfig;

//     use super::*;

//     #[tokio::test]
//     async fn initialize_finds_existing_projector() -> std::io::Result<()> {
//         let temp = tempdir().unwrap();
//         let root_path = temp.into_path();
//         let config = Arc::new(MockConfig::new(root_path));

//         // create a fake projector
//         let mut pidfile = config.paths().projectors().to_owned();
//         pidfile.push("local");
//         fs::create_dir_all(&pidfile)?;
//         pidfile.push("uhura.pid");
//         let mut file = File::create(pidfile)?;
//         file.write_all(format!("{}", std::process::id()).as_bytes())?;

//         let db = Arc::new(MemoryDatabase::new());
//         let repo = Arc::new(MemoryRepository::new(db.clone()));

//         initialize_projectors(config, repo.clone()).await.unwrap();

//         let records = db.projectors().all().unwrap();

//         assert!(!records.is_empty());

//         Ok(())
//     }
// }
