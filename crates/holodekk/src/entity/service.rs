//! Services provide business logic layered on top of the [Repository](super::repository::Repository).
//!
//! Each `Entity` should have an associated Service object that provides the necessary operations
//! (CRUD) required to manage the resource.  It is in these service objects that all business logic
//! should reside.
//!
//! Services are implemented as a series of `Traits` (one for each operation) to ease unit testing
//! in other areas of the system (http endpoints, for example).  Separate traits make mocking much
//! easier, and give a clearer image of the available operations.
//!
//! Each `Trait` based operation should take as input an operation-specific `struct` (even if the
//! operation only accepts a single value) to allow for easier augmentation of service endpoints as
//! development progresses.  For example, fetching a single `Scene` utilizes the following:
//!
//! ```rust,no_run
//! use async_trait::async_trait;
//! use holodekk::entity::service;
//! use holodekk::core::scene;
//!
//! struct Input<'a> {
//!   id: &'a scene::entity::Id,
//! }
//!
//! #[async_trait]
//! trait Get {
//!   async fn get<'a>(&self, input: &'a Input<'a>) -> service::Result<scene::Entity>;
//! }
//! ```
//!
//! The trait can then be mocked in any test that needs to exercise the service.
use super::{id, repository, Id};
use crate::image;

/// Service-related errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The provided Entity Id is formatted incorrectly
    #[error("Invalid Entity ID: {0}")]
    InvalidEntityId(#[from] id::Error),
    /// The provided Image Id is formatted incorrectly
    #[error("Invalid Image ID: {0}")]
    InvalidImageId(#[from] image::id::Error),
    /// The requested resource was not found
    #[error("Entity not found with id {0}")]
    NotFound(Id),
    /// Storing the entity in the repository would cause some sort of duplication
    #[error("Entity already exists")]
    NotUnique(String),
    /// General repository error
    #[error("Repository error occurred")]
    Repository(#[from] repository::Error),
    /// Non-repository unexpected error
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

/// Service operation result type
pub type Result<T> = std::result::Result<T, Error>;
