use std::process::Command;

use log::{debug, error, info, trace, warn};
use nix::{sys::signal::kill, unistd::Pid};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::process::daemon;
use crate::scene;
use crate::utils::fs::ensure_directory;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Message {
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Event {
    Started(i32),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Scene is invalid")]
    InvalidScene,
    #[error("Failure during daemonization")]
    Daemon(#[from] daemon::Error),
    #[error("Error during cleanup")]
    Io(#[from] std::io::Error),
}

pub struct Handle {
    pub sender: Option<Sender<Message>>,
    pub projector_events: Receiver<Event>,
    pub handle: JoinHandle<()>,
}

impl Handle {
    pub async fn stop(mut self) {
        let sender = self.sender.take();
        if let Some(sender) = sender {
            if let Err(err) = sender.send(Message::Shutdown).await {
                error!("Failed to send shutdown message to Scene: {err}");
            }
            drop(sender);
        }
        debug!("Shutdown message sent.  Awiting Scene termination ...");
        if let Err(err) = self.handle.await {
            warn!("Error waiting for Scene termination: {}", err);
        }
        debug!("Scene termination complete.");
    }
}

pub struct Monitor {
    pub id: scene::entity::Id,
    pub name: scene::entity::Name,
    pub status: scene::entity::Status,
    pub scene_paths: scene::Paths,
    pub receiver: Receiver<Message>,
    pub event_sender: Sender<Event>,
    pub paths: crate::Paths,
}

impl Monitor {
    #[must_use]
    pub fn start(paths: crate::Paths, entity: &scene::Entity) -> Handle {
        let (messages_tx, messages_rx) = channel(32);
        let (events_tx, events_rx) = channel(32);
        let scene_id = entity.id.clone();
        let scene_name = entity.name.clone();
        let scene_paths = scene::Paths::build(&paths, &scene_name);

        let handle = tokio::spawn(async move {
            let mut scene = Monitor {
                id: scene_id,
                name: scene_name,
                status: scene::entity::Status::Unknown,
                scene_paths,
                receiver: messages_rx,
                event_sender: events_tx,
                paths,
            };

            scene.run().await;
        });

        Handle {
            sender: Some(messages_tx),
            projector_events: events_rx,
            handle,
        }
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
                        Message::Shutdown => {
                            debug!("Shutting down projector for scene {} ...", self.name);
                            self.stop_projector().unwrap();
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

    async fn check_projector(&mut self) -> std::result::Result<(), Error> {
        let status = if self.scene_paths.pidfile().try_exists().unwrap() {
            let pid = std::fs::read_to_string(self.scene_paths.pidfile())
                .expect("Should have been able to read pid file");
            let pid: i32 = pid
                .parse()
                .expect("Unable to convert pidfile contents to pid");
            match kill(Pid::from_raw(pid), None) {
                Err(_) => {
                    info!(
                        "Found existing pidfile at {}, but no process found",
                        self.scene_paths.pidfile().display()
                    );
                    scene::entity::Status::Crashed
                }
                Ok(_) => scene::entity::Status::Running(pid),
            }
        } else {
            scene::entity::Status::Unknown
        };

        if matches!(status, scene::entity::Status::Running(..)) {
            self.status = status;
            Ok(())
        } else {
            // start it up
            if let scene::entity::Status::Running(pid) =
                self.start_projector(&self.id, &self.name, &self.scene_paths)?
            {
                self.event_sender.send(Event::Started(pid)).await.unwrap();
                self.status = scene::entity::Status::Running(pid);
            }
            Ok(())
        }
    }

    fn start_projector(
        &self,
        id: &scene::entity::Id,
        name: &scene::entity::Name,
        paths: &scene::Paths,
    ) -> std::result::Result<scene::entity::Status, Error> {
        trace!("Scene::start_projector({:?}, {:?}, {:?}", id, name, paths);

        // ensure the root directory exists
        ensure_directory(paths.root()).unwrap();

        // build and execute the actual projector command
        let mut uhura = self.paths.bin_root().clone();
        uhura.push("uhura");

        let mut command = Command::new(uhura);
        command.arg("--id");
        command.arg(id);
        command.arg("--name");
        command.arg(name);

        let pid = daemon::start(&self.paths, command, paths.pidfile())?;
        Ok(scene::entity::Status::Running(pid))
    }

    fn stop_projector(&self) -> std::result::Result<(), Error> {
        trace!("Scene::stop_projector()");
        if let scene::entity::Status::Running(pid) = self.status {
            daemon::stop(pid)?;
            std::fs::remove_dir_all(self.scene_paths.root())?;
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
//                         remove_directory(entry.path()).unwrap();
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
