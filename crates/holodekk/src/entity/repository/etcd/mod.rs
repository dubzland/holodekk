//! Etcd based implementation of the [Repository](super::Repository).

use std::sync::RwLock;

use async_trait::async_trait;
use etcd_client::{Client, Event, WatchStream, Watcher};
use log::{debug, error, warn};
use tokio::sync::broadcast::{channel, Sender};

use crate::entity::{
    self,
    repository::{watch, Error, Result},
};
use crate::scene;

/// Wrapper for managing an instance of a [`WatchTask`].
pub struct WatchTaskHandle<T> {
    watcher: Watcher,
    handle: tokio::task::JoinHandle<()>,
    tx: Sender<T>,
}

/// Processes repository events received from `Etcd`.
///
/// Distributes these events to any subscribers after mapping to the proper internal event.
pub struct WatchTask<T>
where
    T: Send,
{
    stream: WatchStream,
    tx: Sender<T>,
}

impl<T> WatchTask<T>
where
    T: From<Event> + std::fmt::Debug + Clone + Send + 'static,
{
    /// Start a `tokio` task to monitor for incoming `Etcd` events.
    #[must_use]
    pub fn start(watcher: Watcher, stream: WatchStream) -> WatchTaskHandle<T> {
        let (tx, _rx) = channel(32);
        let watch_sender = tx.clone();
        let handle = tokio::spawn(async move {
            let mut watcher = WatchTask {
                stream,
                tx: watch_sender,
            };
            watcher.run().await;
        });

        WatchTaskHandle {
            watcher,
            handle,
            tx,
        }
    }

    async fn run(&mut self) {
        while let Some(resp) = self.stream.message().await.unwrap() {
            if resp.canceled() {
                debug!("Watcher cancelled");
                break;
            }
            for event in resp.events() {
                let res = T::from(event.clone());
                self.tx.send(res).unwrap();
            }
        }
    }
}

/// Scene specific key for  performing get/put operations in `Etcd`.
///
/// Optionally allows a specific id to be appended for managing a specific
/// [`crate::scene::Entity`]
#[must_use]
pub fn scene_key(partial: Option<&entity::Id>) -> String {
    if let Some(partial) = partial {
        format!("/scenes/{partial}")
    } else {
        "/scenes/".to_string()
    }
}

/// Subroutine specific key for  performing get/put operations in `Etcd`.
///
/// Optionally allows a specific id to be appended for managing a specific
/// [`crate::subroutine::Entity`]
#[must_use]
pub fn subroutine_key(partial: Option<&entity::Id>) -> String {
    if let Some(partial) = partial {
        format!("/subroutines/{partial}")
    } else {
        "/subroutines/".to_string()
    }
}

/// `Etcd` based [Repository](super::Repository) implementation.
pub struct Etcd {
    hosts: &'static [&'static str],
    client: RwLock<Option<Client>>,
    scene_watcher: RwLock<Option<WatchTaskHandle<scene::entity::repository::Event>>>,
}

impl Etcd {
    /// Create a new repository instance.
    ///
    /// Note: this function doesn't actually make a connection to `Etcd`.  That is done in
    /// [`entity::Repository::init()`].
    pub fn new(hosts: &'static [&'static str]) -> Self {
        Self {
            hosts,
            client: RwLock::new(None),
            scene_watcher: RwLock::new(None),
        }
    }
}

#[async_trait]
impl entity::Repository for Etcd {
    async fn init(&self) -> Result<()> {
        match Client::connect(self.hosts, None).await {
            Ok(client) => {
                self.client.write().unwrap().replace(client);
                Ok(())
            }
            Err(err) => {
                let msg = format!("Failed to connect to etcd: {err}");
                warn!("{}", msg);
                Err(Error::Initialization(msg))
            }
        }
    }

    async fn shutdown(&self) {
        let scene_watcher = self.scene_watcher.write().unwrap().take();
        if let Some(mut scene_watcher) = scene_watcher {
            scene_watcher.watcher.cancel().await.unwrap();
            scene_watcher.handle.await.unwrap();
        }
    }

    async fn subscribe_scenes(&self) -> Result<watch::Handle<scene::entity::repository::Event>> {
        let have_watcher = self.scene_watcher.read().unwrap().is_some();
        if !have_watcher {
            let mut client = self.client.write().unwrap().clone().unwrap();
            let options = etcd_client::WatchOptions::new()
                .with_prefix()
                .with_prev_key();

            match client.watch(scene_key(None), Some(options)).await {
                Ok((etcd_watcher, stream)) => {
                    let etcd_handle = WatchTask::start(etcd_watcher, stream);
                    self.scene_watcher.write().unwrap().replace(etcd_handle);
                }
                Err(err) => {
                    error!("Failed to setup etcd watcher: {}", err);
                    return Err(Error::Subscribe(err.to_string()));
                }
            };
        }

        let id = watch::Id::generate();
        let rx = self
            .scene_watcher
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .tx
            .subscribe();
        let handle = watch::Handle::new(id, rx);
        Ok(handle)
    }
}

mod scenes;
pub use scenes::*;
mod subroutines;
pub use subroutines::*;
