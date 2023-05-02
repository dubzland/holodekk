use crate::entities::{EntityId, EntityIdError, EntityRepositoryError};
use crate::images::ImageIdError;

#[derive(thiserror::Error, Debug)]
pub enum EntityServiceError {
    #[error("Invalid Entity ID: {0}")]
    InvalidEntityId(#[from] EntityIdError),
    #[error("Invalid Image ID: {0}")]
    InvalidImageId(#[from] ImageIdError),
    #[error("Entity not found with id {0}")]
    NotFound(EntityId),
    #[error("Entity already exists")]
    NotUnique(String),
    #[error("Repository error occurred")]
    Repository(#[from] EntityRepositoryError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub type EntityServiceResult<T> = std::result::Result<T, EntityServiceError>;

pub mod scene;
pub mod subroutine;
