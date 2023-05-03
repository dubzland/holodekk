use super::{id, repository, Id};
use crate::images::ImageIdError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Entity ID: {0}")]
    InvalidEntityId(#[from] id::Error),
    #[error("Invalid Image ID: {0}")]
    InvalidImageId(#[from] ImageIdError),
    #[error("Entity not found with id {0}")]
    NotFound(Id),
    #[error("Entity already exists")]
    NotUnique(String),
    #[error("Repository error occurred")]
    Repository(#[from] repository::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
