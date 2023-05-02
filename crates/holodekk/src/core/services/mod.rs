use crate::core::{
    entities::{EntityId, EntityIdError},
    images::ImageIdError,
    repositories,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Entity ID: {0}")]
    InvalidEntityId(#[from] EntityIdError),
    #[error("Invalid Image ID: {0}")]
    InvalidImageId(#[from] ImageIdError),
    #[error("Entity not found with id {0}")]
    NotFound(EntityId),
    #[error("Entity already exists")]
    NotUnique(String),
    #[error("Repository error occurred")]
    Repository(#[from] repositories::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod scene;
pub mod subroutine;
