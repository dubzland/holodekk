mod scene;
pub use scene::*;
mod subroutine;
pub use subroutine::*;
mod watch;
pub use watch::*;

use async_trait::async_trait;

use super::entities::EntityId;
use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Error initializing repository: {0}")]
    Initialization(String),
    #[error("General repository error: {0}")]
    General(String),
    #[error("Record not found: {0}")]
    NotFound(EntityId),
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

pub trait RepositoryQuery: Send + Sized + Sync {
    type Entity: Sized;

    fn matches(&self, record: &Self::Entity) -> bool;
}

#[async_trait]
pub trait Repository:
    scene::ScenesRepository + subroutine::SubroutinesRepository + 'static
{
    //+ subroutine::SubroutinesRepository {
    async fn init(&self) -> Result<()>;
    async fn shutdown(&self);
    async fn subscribe_scenes(&self) -> Result<watch::WatchHandle<SceneEvent>>;
}

#[cfg(test)]
pub mod fixtures {
    use async_trait::async_trait;
    use mockall::mock;
    use rstest::*;

    use super::{
        MockScenesRepository, MockSubroutinesRepository, Result, ScenesQuery, ScenesRepository,
        SubroutinesQuery, SubroutinesRepository,
    };
    use crate::core::entities::{
        SceneEntity, SceneEntityId, SceneName, SubroutineEntity, SubroutineEntityId,
    };
    use crate::core::enums::{SceneStatus, SubroutineStatus};

    #[fixture]
    pub fn mock_scenes_repository() -> MockScenesRepository {
        MockScenesRepository::default()
    }

    #[fixture]
    pub fn mock_subroutines_repository() -> MockSubroutinesRepository {
        MockSubroutinesRepository::default()
    }

    mock! {
        pub Repository {}

        #[async_trait]
        impl ScenesRepository for Repository {
            async fn scenes_create(
                &self,
                scene: SceneEntity,
            ) -> Result<SceneEntity>;
            async fn scenes_delete(&self, id: &SceneEntityId) -> Result<()>;
            async fn scenes_exists<'a>(&self, query: ScenesQuery<'a>) -> Result<bool>;
            async fn scenes_find<'a>(&self, query: ScenesQuery<'a>)
                -> Result<Vec<SceneEntity>>;
            async fn scenes_get(&self, id: &SceneEntityId) -> Result<SceneEntity>;
            async fn scenes_update(&self, id: &SceneEntityId, name: Option<SceneName>, status: Option<SceneStatus>) -> Result<SceneEntity>;
        }

        #[async_trait]
        impl SubroutinesRepository for Repository {
            async fn subroutines_create(
                &self,
                subroutine: SubroutineEntity,
            ) -> Result<SubroutineEntity>;
            async fn subroutines_delete(&self, id: &SubroutineEntityId) -> Result<()>;
            async fn subroutines_exists<'a>(&self, query: SubroutinesQuery<'a>) -> Result<bool>;
            async fn subroutines_find<'a>(
                &self,
                query: SubroutinesQuery<'a>,
            ) -> Result<Vec<SubroutineEntity>>;
            async fn subroutines_get(&self, id: &SubroutineEntityId) -> Result<SubroutineEntity>;
            async fn subroutines_update(&self, id: &SubroutineEntityId, status: Option<SubroutineStatus>) -> Result<SubroutineEntity>;
        }
    }

    #[fixture]
    pub(crate) fn mock_repository() -> MockRepository {
        MockRepository::default()
    }
}
