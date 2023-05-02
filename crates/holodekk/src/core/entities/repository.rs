use async_trait::async_trait;
use log::warn;
use tokio::sync::broadcast::{error::RecvError, Receiver};

use crate::errors::error_chain_fmt;

use super::{EntityId, SceneEntityRepositoryEvent};

#[derive(thiserror::Error)]
pub enum EntityRepositoryError {
    #[error("Error initializing repository: {0}")]
    Initialization(String),
    #[error("General repository error: {0}")]
    General(String),
    #[error("Record not found: {0}")]
    NotFound(EntityId),
    #[error("Entity conflict: {0}")]
    Conflict(String),
    #[error("Failed to setup subscription: {0}")]
    Subscribe(String),
    #[error("Etcd communication error")]
    Etcd(#[from] etcd_client::Error),
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}

impl std::fmt::Debug for EntityRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type EntityRepositoryResult<T> = std::result::Result<T, EntityRepositoryError>;

pub trait EntityRepositoryQuery: Send + Sized + Sync {
    type Entity: Sized;

    fn matches(&self, record: &Self::Entity) -> bool;
}

pub type EntityRepositoryWatchId = EntityId;

pub enum EntityRepositoryWatchError {}

pub struct EntityRepositoryWatchHandle<T> {
    pub id: EntityRepositoryWatchId,
    pub rx: Receiver<T>,
}

impl<T> EntityRepositoryWatchHandle<T>
where
    T: Clone,
{
    pub fn new(id: EntityRepositoryWatchId, rx: Receiver<T>) -> Self {
        Self { id, rx }
    }

    pub async fn event(&mut self) -> Option<T> {
        match self.rx.recv().await {
            Ok(msg) => Some(msg),
            Err(RecvError::Closed) => None,
            Err(err) => {
                warn!("Error receiving watch event: {}", err);
                None
            }
        }
    }
}

#[async_trait]
pub trait EntityRepository:
    super::scene::SceneEntityRepository + super::subroutine::SubroutineEntityRepository + 'static
{
    async fn init(&self) -> EntityRepositoryResult<()>;
    async fn shutdown(&self);
    async fn subscribe_scenes(
        &self,
    ) -> EntityRepositoryResult<EntityRepositoryWatchHandle<SceneEntityRepositoryEvent>>;
}
