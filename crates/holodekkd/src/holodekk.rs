use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, info, trace, warn};
use nix::{sys::signal::kill, unistd::Pid};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use holodekk::core::{
    entities::{
        repository::{Repository, WatchHandle},
        SceneEntity, SceneEvent, SceneName,
    },
    enums::SceneStatus,
    services::scene::{FindScenes, ScenesFindInput, ScenesService},
    ScenePaths,
};
use holodekk::utils::process::terminate_daemon;

use super::scene::{Scene, SceneError, SceneHandle};
use crate::config::HolodekkdConfig;

#[derive(Debug)]
pub enum HolodekkMessage {}

#[derive(thiserror::Error, Debug)]
pub enum HolodekkError {
    #[error("Scene error")]
    Scene(#[from] SceneError),
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Error during Holodekk initialization: {0}")]
    Initialization(String),
}

pub enum HolodekkEvent {}

pub struct HolodekkHandle<R> {
    pub sender: Option<Sender<HolodekkMessage>>,
    pub event_receiver: Receiver<HolodekkEvent>,
    pub handle: JoinHandle<()>,
    pub repo: Arc<R>,
}

impl<R> HolodekkHandle<R>
where
    R: Repository,
{
    pub async fn stop(mut self) {
        let sender = self.sender.take().unwrap();
        drop(sender);
        self.repo.shutdown().await;
        self.handle.await.unwrap();
    }
}

pub struct Holodekk<R>
where
    R: Repository,
{
    pub scenes: HashMap<SceneName, SceneHandle>,
    pub receiver: Receiver<HolodekkMessage>,
    pub event_sender: Sender<HolodekkEvent>,
    pub scene_watcher: WatchHandle<SceneEvent>,
    pub config: Arc<HolodekkdConfig>,
    pub repo: Arc<R>,
}

impl<R> Holodekk<R>
where
    R: Repository,
{
    pub async fn start(
        config: Arc<HolodekkdConfig>,
        repo: Arc<R>,
    ) -> std::result::Result<HolodekkHandle<R>, HolodekkError> {
        let (messages_tx, messages_rx) = channel(32);
        let (events_tx, events_rx) = channel(32);

        let scenes = initialize_scenes(config.clone(), repo.clone()).await?;

        let scene_watcher = repo.subscribe_scenes().await.unwrap();

        let handle = {
            let repo = repo.clone();
            tokio::spawn(async move {
                let mut holodekk = Holodekk {
                    config,
                    scenes,
                    receiver: messages_rx,
                    event_sender: events_tx,
                    scene_watcher,
                    repo,
                };

                holodekk.run().await;
            })
        };

        Ok(HolodekkHandle {
            sender: Some(messages_tx),
            event_receiver: events_rx,
            handle,
            repo,
        })
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(message) = self.receiver.recv() => {
                    trace!("message from holodekk receiver: {:?}", message);
                }
                Some(event) = self.scene_watcher.event() => {
                    trace!("Scene update from repo: {:?}", event);
                    match event {
                        SceneEvent::Unknown => {},
                        SceneEvent::Insert { scene } => {
                            trace!("I want to start a scene: {:?}", scene);
                            self.create_scene(&scene).await.unwrap();
                        }
                        SceneEvent::Update { .. } => {}
                        SceneEvent::Delete { scene } => {
                            self.destroy_scene(&scene).await.unwrap();
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

    pub async fn create_scene(&mut self, entity: &SceneEntity) -> Result<(), HolodekkError> {
        let scene = Scene::start(self.config.clone(), entity).await?;
        self.scenes.insert(entity.name.to_owned(), scene);
        Ok(())
    }

    pub async fn destroy_scene(&mut self, entity: &SceneEntity) -> Result<(), HolodekkError> {
        if let Some(scene) = self.scenes.remove(&entity.name) {
            scene.stop().await?;
        }
        Ok(())
    }
}

pub async fn initialize_scenes<R>(
    config: Arc<HolodekkdConfig>,
    repo: Arc<R>,
) -> Result<HashMap<SceneName, SceneHandle>, HolodekkError>
where
    R: Repository,
{
    let mut scenes = HashMap::new();

    let scenes_service = ScenesService::new(repo.clone());

    // get the list of scenes from repository
    let mut repo_scenes = scenes_service
        .find(&ScenesFindInput::default())
        .await
        .map_err(|err| HolodekkError::Initialization(format!("{:?}", err)))?;

    // get the list of actually running scenes
    let mut running_scenes: Vec<SceneEntity> = std::fs::read_dir(config.paths().scenes_root())
        .unwrap()
        .filter_map(|e| {
            let entry = e.unwrap();
            let name: SceneName = entry.path().as_path().file_name().unwrap().to_str().unwrap().into();
            let paths = ScenePaths::build(config.paths(), &name);
            if paths.pidfile().try_exists().unwrap() {
                let pid = std::fs::read_to_string(paths.pidfile())
                    .expect("Should have been able to read pid file");
                let pid: i32 = pid
                    .parse()
                    .expect("Unable to convert pidfile contents to pid");
                match kill(Pid::from_raw(pid), None) {
                    Err(_) => {
                        info!(
                            "Found existing pidfile at {}, but no process found. Removing directory",
                            paths.pidfile().display()
                        );
                        warn!("Removing directory: {}", entry.path().display());
                        std::fs::remove_dir_all(entry.path()).unwrap();
                        None
                    }
                    Ok(_) => {
                        debug!("Initializing scene for existing projector process: {}", name);
                        let mut s = SceneEntity::new(name);
                        s.status = SceneStatus::Running(pid);
                        Some(s)
                    }
                }
            } else {
                None
            }
        })
        .collect();

    // synchronize
    while let Some(repo_scene) = repo_scenes.pop() {
        let scene = Scene::start(config.clone(), &repo_scene).await.unwrap();
        scenes.insert(repo_scene.name.clone(), scene);
        if let Some(idx) = running_scenes
            .iter()
            .position(|r| r.status == repo_scene.status || r.name == repo_scene.name)
        {
            running_scenes.remove(idx);
        }
    }

    // at this point, anything still in running isn't valid.  trash it.
    for scene in running_scenes {
        debug!("cleaning up dead scene: {}", scene.name);
        if let SceneStatus::Running(pid) = scene.status {
            terminate_daemon(pid).unwrap();
        }
    }

    Ok(scenes)
}
