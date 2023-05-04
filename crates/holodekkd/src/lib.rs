use std::collections::HashMap;
use std::fs::DirEntry;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use nix::{sys::signal::kill, unistd::Pid};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use holodekk::entity::repository;
use holodekk::process::daemon;
use holodekk::scene::{self, entity::service::Find};

#[derive(Debug)]
pub enum Message {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Scene error")]
    Scene(#[from] scene::monitor::Error),
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Error during Holodekk initialization: {0}")]
    Initialization(String),
    #[error("Repository error occurred")]
    Repository(#[from] holodekk::entity::repository::Error),
}

pub enum Event {}

pub struct Handle<R> {
    pub sender: Option<Sender<Message>>,
    pub event_receiver: Receiver<Event>,
    pub handle: JoinHandle<()>,
    pub repo: Arc<R>,
}

impl<R> Handle<R>
where
    R: holodekk::entity::Repository,
{
    pub async fn stop(mut self) {
        if let Some(sender) = self.sender.take() {
            drop(sender);
        }
        self.repo.shutdown().await;
        if let Err(err) = self.handle.await {
            warn!("Error encountered waiting for Holodekk shutdown: {err}");
        }
    }
}

pub struct Monitor<R>
where
    R: holodekk::entity::Repository,
{
    pub scenes: HashMap<scene::entity::Name, scene::monitor::Handle>,
    pub receiver: Receiver<Message>,
    pub event_sender: Sender<Event>,
    pub scene_watcher: repository::watch::Handle<scene::entity::repository::Event>,
    pub config: crate::Config,
    pub repo: Arc<R>,
}

impl<R> Monitor<R>
where
    R: holodekk::entity::Repository,
{
    /// # Errors
    ///
    /// - Scene initialization fails (repository or filesystem error)
    /// - Setting up the repository watcher for scenes fails
    pub async fn start(
        config: crate::Config,
        repo: Arc<R>,
    ) -> std::result::Result<Handle<R>, Error> {
        let (messages_tx, messages_rx) = channel(32);
        let (events_tx, events_rx) = channel(32);

        let scenes = initialize_scenes(&config, repo.clone()).await?;

        let scene_watcher = repo.subscribe_scenes().await?;

        let handle = {
            let repo = repo.clone();
            tokio::spawn(async move {
                let mut holodekk = Monitor {
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

        Ok(Handle {
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
                        scene::entity::repository::Event::Unknown | scene::entity::repository::Event::Update { .. } => {},
                        scene::entity::repository::Event::Insert { scene } => {
                            trace!("I want to start a scene: {:?}", scene);
                            self.create_scene(&scene);
                        }
                        scene::entity::repository::Event::Delete { scene } => {
                            self.destroy_scene(&scene).await;
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

    pub fn create_scene(&mut self, entity: &scene::Entity) {
        let scene = scene::Monitor::start(self.config.paths().clone(), entity);
        self.scenes.insert(entity.name.clone(), scene);
    }

    pub async fn destroy_scene(&mut self, entity: &scene::Entity) {
        if let Some(scene) = self.scenes.remove(&entity.name) {
            scene.stop().await;
        }
    }
}

/// # Errors
///
/// Will return `Err` in case of repository failure, or if the scenes directory is inaccessible.
pub async fn initialize_scenes<R>(
    config: &crate::Config,
    repo: Arc<R>,
) -> Result<HashMap<scene::entity::Name, scene::monitor::Handle>, Error>
where
    R: holodekk::entity::Repository,
{
    let mut scenes = HashMap::new();

    // get the list of scenes from repository
    let mut repo_scenes = get_repo_scenes(repo).await?;

    // get the list of actually running scenes
    let mut running_scenes = get_running_scenes(config)?;

    // synchronize
    while let Some(repo_scene) = repo_scenes.pop() {
        let scene = scene::Monitor::start(config.paths().clone(), &repo_scene);
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
        if let scene::entity::Status::Running(pid) = scene.status {
            if let Err(err) = daemon::stop(pid) {
                warn!("failed to terminate projector running with pid {pid}: {err}");
            }
        }
    }

    Ok(scenes)
}

async fn get_repo_scenes<R>(repo: Arc<R>) -> std::result::Result<Vec<scene::Entity>, Error>
where
    R: holodekk::entity::Repository,
{
    let scenes_service = scene::entity::Service::new(repo.clone());

    scenes_service
        .find(&scene::entity::service::find::Input::default())
        .await
        .map_err(|err| Error::Initialization(format!("{err:?}")))
}

fn get_running_scenes(config: &crate::Config) -> std::result::Result<Vec<scene::Entity>, Error> {
    match std::fs::read_dir(config.paths().scenes_root()) {
        Ok(entries) => {
            let scenes = entries
                .filter_map(|readdir| match readdir {
                    Err(err) => {
                        let msg = format!("failed to iterate entry: {err}");
                        warn!("{msg}");
                        None
                    }
                    Ok(entry) => validate_scene_path(&entry, config),
                })
                .collect();
            Ok(scenes)
        }
        Err(err) => {
            let msg = format!("failed to list scenes directory: {err}");
            error!("{err}");
            Err(Error::Initialization(msg))
        }
    }
}

fn validate_scene_path(entry: &DirEntry, config: &crate::Config) -> Option<scene::Entity> {
    if let Some(filename) = entry.file_name().to_str() {
        let name = filename.into();
        let paths = scene::Paths::build(config.paths(), &name);

        if let Ok(exists) = paths.pidfile().try_exists() {
            if exists {
                let pid = std::fs::read_to_string(paths.pidfile())
                    .expect("Should have been able to read pid file");
                let pid: i32 = pid
                    .parse()
                    .expect("Unable to convert pidfile contents to pid");
                if kill(Pid::from_raw(pid), None).is_err() {
                    info!(
                        "Found existing pidfile at {}, but no process found. Removing directory",
                        paths.pidfile().display()
                    );
                    warn!("Removing directory: {}", entry.path().display());
                    if let Err(err) = std::fs::remove_dir_all(entry.path()) {
                        warn!("Failed to cleanup directory {:?}: {}", entry.path(), err);
                    }
                    None
                } else {
                    debug!(
                        "Initializing scene for existing projector process: {}",
                        name
                    );
                    let mut s = scene::Entity::new(name);
                    s.status = scene::entity::Status::Running(pid);
                    Some(s)
                }
            } else {
                None
            }
        } else {
            warn!(
                "Failed to check the existence of pidfile: {:?}",
                paths.pidfile()
            );
            None
        }
    } else {
        warn!("Failed to convert {:?} to_str", entry.file_name());
        None
    }
}

pub mod api;
mod config;
pub use config::*;
