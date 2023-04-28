pub mod etcd;
pub mod memory;

use clap::ValueEnum;

use crate::core::entities::EntityId;

#[derive(thiserror::Error, Debug)]
pub enum RepositoryError {
    #[error("General repository error: {0}")]
    General(String),
    #[error("Record not found: {0}")]
    NotFound(EntityId),
    #[error("Entity conflict: {0}")]
    Conflict(String),
    #[error("Etcd communication error")]
    Etcd(#[from] etcd_client::Error),
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum RepositoryKind {
    Memory,
}

pub trait RepositoryId {
    fn id(&self) -> String;
}

pub trait RepositoryQuery: Send + Sized + Sync {
    type Entity: Sized;

    fn matches(&self, record: &Self::Entity) -> bool;
}
