//! Entity service for the `scene` [`Repository`].

use std::sync::Arc;

use crate::scene::entity::Repository;

/// Shortcut supertrait for expressing full service implementation.
pub trait Methods: Create + Delete + Find + Get {}
impl<T> Methods for T where T: Create + Delete + Find + Get {}

/// Service for interacting with the `Scene` entity [`repository`][`Repository`].
#[derive(Debug)]
pub struct Service<R>
where
    R: Repository,
{
    repo: Arc<R>,
}

impl<R> Service<R>
where
    R: Repository,
{
    /// Wrap the given repository in a service instance.
    #[must_use]
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

/// Store a new [`Entity`][`crate::scene::Entity`] in the repository.
pub mod create;
pub use create::Create;
/// Delete an existing [`Entity`][`crate::scene::Entity`] from the repository.
pub mod delete;
pub use delete::Delete;
/// Find one or more [`Entities`][`crate::scene::Entity`] in the repository.
pub mod find;
pub use find::Find;
/// Retrieve a single [`Entity`][`crate::scene::Entity`] from the repository.
pub mod get;
pub use get::Get;

#[cfg(test)]
pub mod fixtures {
    use async_trait::async_trait;
    use mockall::mock;
    use rstest::*;

    use crate::entity;
    use crate::scene;

    pub use super::create::MockCreate;
    pub use super::delete::MockDelete;
    pub use super::find::MockFind;
    pub use super::get::MockGet;
    use super::*;

    mock! {
        pub Service {}
        #[async_trait]
        impl Create for Service {
            async fn create<'a>(&self, input: &'a super::create::Input<'a>) -> entity::service::Result<scene::Entity>;
        }

        #[async_trait]
        impl Delete for Service {
            async fn delete<'a>(&self, input: &'a super::delete::Input<'a>) -> entity::service::Result<()>;
        }

        #[async_trait]
        impl Find for Service {
            async fn find<'a>(&self, input: &'a super::find::Input<'a>) -> entity::service::Result<Vec<scene::Entity>>;
        }

        #[async_trait]
        impl Get for Service {
            async fn get<'a>(&self, input: &'a super::get::Input<'a>) -> entity::service::Result<scene::Entity>;
        }
    }

    #[fixture]
    pub fn mock_create() -> MockCreate {
        MockCreate::default()
    }

    #[fixture]
    pub fn mock_delete() -> MockDelete {
        MockDelete::default()
    }

    #[fixture]
    pub fn mock_find() -> MockFind {
        MockFind::default()
    }

    #[fixture]
    pub fn mock_get() -> MockGet {
        MockGet::default()
    }

    #[fixture]
    pub fn mock_service() -> MockService {
        MockService::default()
    }
}

#[cfg(test)]
pub use fixtures::*;
