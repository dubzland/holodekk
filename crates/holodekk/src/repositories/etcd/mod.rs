mod scenes;
pub use scenes::*;
mod subroutines;
pub use subroutines::*;

use std::sync::RwLock;

use async_trait::async_trait;
use etcd_client::{Client, Event, WatchStream, Watcher};
use log::{debug, error, warn};
use tokio::sync::broadcast::{channel, Sender};

use crate::core::{
    entities::EntityId,
    repositories::{Error, Repository, Result, SceneEvent, WatchHandle, WatchId},
};

pub struct EtcdWatchHandle<T> {
    watcher: Watcher,
    handle: tokio::task::JoinHandle<()>,
    tx: Sender<T>,
}

pub struct EtcdWatcher<T>
where
    T: Send,
{
    stream: WatchStream,
    tx: Sender<T>,
}

impl<T> EtcdWatcher<T>
where
    T: From<Event> + std::fmt::Debug + Clone + Send + 'static,
{
    pub fn start(watcher: Watcher, stream: WatchStream) -> EtcdWatchHandle<T> {
        let (tx, _rx) = channel(32);
        let watch_sender = tx.clone();
        let handle = tokio::spawn(async move {
            let mut watcher = EtcdWatcher {
                stream,
                tx: watch_sender,
            };
            watcher.run().await;
        });

        EtcdWatchHandle {
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
                let res = T::from(event.to_owned());
                self.tx.send(res).unwrap();
            }
        }
    }
}

pub fn etcd_scene_key(partial: Option<&EntityId>) -> String {
    if let Some(partial) = partial {
        format!("/scenes/{}", partial)
    } else {
        "/scenes/".to_string()
    }
}

pub fn etcd_subroutine_key(partial: Option<&EntityId>) -> String {
    if let Some(partial) = partial {
        format!("/subroutines/{}", partial)
    } else {
        "/subroutines/".to_string()
    }
}

pub struct EtcdRepository {
    hosts: &'static [&'static str],
    client: RwLock<Option<Client>>,
    scene_watcher: RwLock<Option<EtcdWatchHandle<SceneEvent>>>,
}

impl EtcdRepository {
    pub fn new(hosts: &'static [&'static str]) -> Self {
        Self {
            hosts,
            client: RwLock::new(None),
            scene_watcher: RwLock::new(None),
        }
    }
}

#[async_trait]
impl Repository for EtcdRepository {
    async fn init(&self) -> std::result::Result<(), Error> {
        match Client::connect(self.hosts, None).await {
            Ok(client) => {
                self.client.write().unwrap().replace(client);
                Ok(())
            }
            Err(err) => {
                let msg = format!("Failed to connect to etcd: {}", err);
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

    async fn subscribe_scenes(&self) -> Result<WatchHandle<SceneEvent>> {
        let have_watcher = self.scene_watcher.read().unwrap().is_some();
        if !have_watcher {
            let mut client = self.client.write().unwrap().clone().unwrap();
            let options = etcd_client::WatchOptions::new()
                .with_prefix()
                .with_prev_key();

            match client.watch(etcd_scene_key(None), Some(options)).await {
                Ok((etcd_watcher, stream)) => {
                    let etcd_handle = EtcdWatcher::start(etcd_watcher, stream);
                    self.scene_watcher.write().unwrap().replace(etcd_handle);
                }
                Err(err) => {
                    error!("Failed to setup etcd watcher: {}", err);
                    return Err(Error::Subscribe(err.to_string()));
                }
            };
        }

        let id = WatchId::generate();
        let rx = self
            .scene_watcher
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .tx
            .subscribe();
        let handle = WatchHandle::new(id, rx);
        Ok(handle)
    }
}
