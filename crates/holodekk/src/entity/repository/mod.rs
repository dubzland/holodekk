use async_trait::async_trait;
use clap::ValueEnum;

use crate::errors::error_chain_fmt;
use crate::scene;
use crate::subroutine;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Kind {
    Etcd,
    Memory,
}

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Error initializing repository: {0}")]
    Initialization(String),
    #[error("General repository error: {0}")]
    General(String),
    #[error("Record not found: {0}")]
    NotFound(crate::entity::Id),
    #[error("Entity conflict: {0}")]
    Conflict(String),
    #[error("Failed to setup subscription: {0}")]
    Subscribe(String),
    #[error("Etcd communication error")]
    Etcd(#[from] etcd_client::Error),
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Query: Send + Sized + Sync {
    type Entity: Sized;

    fn matches(&self, record: &Self::Entity) -> bool;
}

#[async_trait]
pub trait Repository: scene::entity::Repository + subroutine::entity::Repository + 'static {
    async fn init(&self) -> Result<()>;
    async fn shutdown(&self);
    async fn subscribe_scenes(&self) -> Result<watch::Handle<scene::entity::repository::Event>>;
}

pub mod watch;

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use async_trait::async_trait;
    use mockall::mock;

    use crate::scene::{self, entity::Repository as SceneRepository};
    use crate::subroutine::{self, entity::Repository as SubroutineRepository};

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
