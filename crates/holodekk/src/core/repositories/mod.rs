pub mod memory;

use clap::ValueEnum;

#[derive(thiserror::Error, Clone, Copy, Debug, PartialEq)]
pub enum Error {
    #[error("General Error")]
    General,
    #[error("Entity not found")]
    NotFound,
    #[error("Record already exists")]
    AlreadyExists,
    #[error("Relation not found")]
    RelationNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

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
