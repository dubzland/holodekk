pub mod memory;

use clap::ValueEnum;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum RepositoryError {
    #[error("General repository error: {0}")]
    General(String),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Record already exists with id {0}")]
    Duplicate(String),
}

pub type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum RepositoryKind {
    Memory,
}

pub trait RepositoryId {
    fn id(&self) -> String;
}

pub trait RepositoryQuery: Send + Sized {
    type Entity: Sized;

    fn matches(&self, record: &Self::Entity) -> bool;
}
