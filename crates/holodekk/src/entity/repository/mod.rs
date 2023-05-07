//! Abstract repository definition to be implemented by backing stores.
//!
//! All `Entities` within the `Holodekk` are persisted via `Repository` instances, and most of the
//! system is built as generics over the type of repository used.  However, almost all `Repository`
//! interaction should take place via a `Service` object to allow all requests to flow through
//! consistent business logic.
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::sync::Arc;
//!
//! use holodekk::entity::repository::{Kind, Repository, Etcd, memory::Database, Memory};
//!
//! # #[tokio::main]
//! # async fn main() {
//! // load the repo_kind from cli options
//! # let repo_kind = Kind::Memory;
//! match repo_kind {
//!    Kind::Memory => {
//!      let database = Arc::new(Database::new());
//!      let repo = Memory::new(database);
//!      run_server(repo)
//!    }
//!    Kind::Etcd => {
//!      let repo = Etcd::new(&["127.0.0.1:3279"]);
//!      run_server(repo)
//!    }
//! };
//! # fn run_server<R>(repo: R) where R: Repository {}
//! # }
use async_trait::async_trait;
use clap::ValueEnum;

use crate::core::scene;
use crate::core::subroutine;
use crate::errors::error_chain_fmt;

/// Type of repository currently being used
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Kind {
    /// Etcd-backed repository
    Etcd,
    /// Memory repository (for testing)
    Memory,
}

/// Errors encountered during `Repository` operation
#[derive(thiserror::Error)]
pub enum Error {
    /// Failure occurred during repository initilization (connection issues usually)
    #[error("Error initializing repository: {0}")]
    Initialization(String),
    /// Failed to find an Entity by Id
    #[error("Record not found: {0}")]
    NotFound(crate::entity::Id),
    /// Supplied data would cause a conflict (duplicate)
    #[error("Entity conflict: {0}")]
    Conflict(String),
    /// Failure setting up a repository Watch
    #[error("Failed to setup subscription: {0}")]
    Subscribe(String),
    /// Unspecified etcd client communications error
    #[error("Etcd communication error")]
    Etcd(#[from] etcd_client::Error),
    /// JSON serialization/deserialization failure
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Repository operation result type
pub type Result<T> = std::result::Result<T, Error>;

/// Abstract `Repository` Query
///
/// Since this isn't a very filter-heavy application, simple boolean matching serves the purpose.
pub trait Query: Send + Sized + Sync {
    /// The entity associated with this query object
    type Entity: Sized;

    /// Provides a mechanism to assert wither a given record "matches" a particular query, and
    /// should be returned to the caller.
    fn matches(&self, record: &Self::Entity) -> bool;
}

/// Base repository methods
#[async_trait]
pub trait Repository: scene::entity::Repository + subroutine::entity::Repository + 'static {
    /// Repository initialization.
    ///
    /// This is where connections should be established and caches should be primed.
    async fn init(&self) -> Result<()>;

    /// Repository teardown.
    ///
    /// Close connections and persist caches.
    async fn shutdown(&self);

    /// Setup a watch on scenes.
    async fn subscribe_scenes(&self) -> Result<watch::Handle<scene::entity::repository::Event>>;
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use async_trait::async_trait;
    use mockall::mock;

    use crate::core::scene::{self, entity::Repository as SceneRepository};
    use crate::core::subroutine::{self, entity::Repository as SubroutineRepository};

    use super::*;

    mock! {
        pub Repository {}

        #[async_trait]
        impl SceneRepository for Repository {
            async fn scenes_create(
                &self,
                scene: scene::Entity,
            ) -> Result<scene::Entity>;
            async fn scenes_delete(&self, id: &scene::entity::Id) -> Result<()>;
            async fn scenes_exists<'a>(&self, query: scene::entity::repository::Query<'a>) -> Result<bool>;
            async fn scenes_find<'a>(&self, query: scene::entity::repository::Query<'a>)
                -> Result<Vec<scene::Entity>>;
            async fn scenes_get(&self, id: &scene::entity::Id) -> Result<scene::Entity>;
            async fn scenes_update(&self, id: &scene::entity::Id, name: Option<scene::entity::Name>, status: Option<scene::entity::Status>) -> Result<scene::Entity>;
        }

        #[async_trait]
        impl SubroutineRepository for Repository {
            async fn subroutines_create(
                &self,
                subroutine: subroutine::Entity,
            ) -> Result<subroutine::Entity>;
            async fn subroutines_delete(&self, id: &subroutine::entity::Id) -> Result<()>;
            async fn subroutines_exists<'a>(&self, query: subroutine::entity::repository::Query<'a>) -> Result<bool>;
            async fn subroutines_find<'a>(
                &self,
                query: subroutine::entity::repository::Query<'a>,
            ) -> Result<Vec<subroutine::Entity>>;
            async fn subroutines_get(&self, id: &subroutine::entity::Id) -> Result<subroutine::Entity>;
            async fn subroutines_update(&self, id: &subroutine::entity::Id, status: Option<subroutine::entity::Status>) -> Result<subroutine::Entity>;
        }
    }

    #[fixture]
    pub fn mock_repository() -> MockRepository {
        MockRepository::default()
    }
}

#[cfg(test)]
pub use fixtures::*;

pub mod etcd;
pub use etcd::Etcd;
pub mod memory;
pub use memory::Memory;
pub mod watch;
