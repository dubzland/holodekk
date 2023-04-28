use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

pub enum SceneMessage {}
pub enum SceneEvent {}

#[derive(thiserror::Error, Debug)]
pub enum SceneError {}

pub struct SceneHandle {
    pub sender: Option<Sender<SceneMessage>>,
    pub projector_events: Receiver<SceneEvent>,
    pub handle: JoinHandle<()>,
}

impl SceneHandle {
    pub async fn stop(mut self) -> Result<(), SceneError> {
        let sender = self.sender.take().unwrap();
        drop(sender);
        self.handle.await.unwrap();
        Ok(())
    }
}

pub struct Scene {
    pub name: String,
    pub sender: Sender<SceneMessage>,
    pub receiver: Receiver<SceneMessage>,
    pub event_sender: Sender<SceneEvent>,
}

impl Scene {
    pub async fn start(name: &str) -> std::result::Result<SceneHandle, SceneError> {
        let (messages_tx, messages_rx) = channel(32);
        let (events_tx, events_rx) = channel(32);
        let scene_name = name.to_string();

        let projector_sender = messages_tx.clone();
        let handle = tokio::spawn(async move {
            let mut scene = Scene {
                name: scene_name,
                sender: projector_sender,
                receiver: messages_rx,
                event_sender: events_tx,
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
        // check for scene process, and start if not running
        // monitor events
        todo!()
    }
}

// async fn process_spawn_request(
//     &self,
//     mut projector: ProjectorEntity,
// ) -> std::result::Result<ProjectorEntity, ProjectorSpawnError> {
//     trace!("ProjectorsWorker::process_spawn_request({:?})", projector);

//     let projector_paths = ProjectorPaths::build(self.paths.clone(), &projector);

//     // ensure the root directory exists
//     ensure_directory(projector_paths.root())?;

//     // build and execute the actual projector command
//     let mut uhura = self.paths.bin_root().clone();
//     uhura.push("uhura");

//     let mut command = Command::new(uhura);
//     command.arg("--name");
//     command.arg(projector.name());

//     let pid = daemonize(self.paths.clone(), command, projector_paths.pidfile())?;

//     projector.set_status(ProjectorStatus::Running(pid as u32));
//     Ok(projector)
// }

// async fn process_terminate_request(
//     &self,
//     projector: &ProjectorEntity,
// ) -> std::result::Result<(), ProjectorTerminationError> {
//     trace!(
//         "ProjectorsWorker::process_terminate_request({:?})",
//         projector
//     );
//     if let ProjectorStatus::Running(pid) = projector.status() {
//         terminate_daemon(pid as i32)?;

//         let projector_paths = ProjectorPaths::build(self.paths.clone(), &projector);

//         std::fs::remove_dir_all(projector_paths.root())?;
//         debug!("Projector cleanup complete.");
//     }

//     Ok(())
// }

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
