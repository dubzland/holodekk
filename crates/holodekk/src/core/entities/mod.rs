mod id;
pub use id::*;
mod scene;
pub use scene::*;
mod subroutine;
pub use subroutine::*;

pub mod repository {

    use async_trait::async_trait;
    use log::warn;
    use tokio::sync::broadcast::{error::RecvError, Receiver};

    use crate::errors::error_chain_fmt;

    use super::{EntityId, SceneEvent};

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

    pub type WatchId = EntityId;

    pub enum WatchError {}

    pub struct WatchHandle<T> {
        pub id: WatchId,
        pub rx: Receiver<T>,
    }

    impl<T> WatchHandle<T>
    where
        T: Clone,
    {
        pub fn new(id: WatchId, rx: Receiver<T>) -> Self {
            Self { id, rx }
        }

        pub async fn event(&mut self) -> Option<T> {
            match self.rx.recv().await {
                Ok(msg) => Some(msg),
                Err(RecvError::Closed) => None,
                Err(err) => {
                    warn!("Error receiving watch event: {}", err);
                    None
                }
            }
        }
    }

    #[async_trait]
    pub trait Repository:
        super::scene::ScenesRepository + super::subroutine::SubroutinesRepository + 'static
    {
        async fn init(&self) -> Result<()>;
        async fn shutdown(&self);
        async fn subscribe_scenes(&self) -> Result<WatchHandle<SceneEvent>>;
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use async_trait::async_trait;
    use mockall::mock;

    use super::repository::Result;
    use super::{
        MockScenesRepository, MockSubroutinesRepository, ScenesQuery, ScenesRepository,
        SubroutinesQuery, SubroutinesRepository,
    };

    use crate::core::enums::{SceneStatus, SubroutineStatus};
    use crate::core::images::{fixtures::mock_subroutine_image, SubroutineImage};

    use super::*;

    #[fixture]
    pub fn mock_scene_entity() -> SceneEntity {
        SceneEntity::new("test".into())
    }

    #[fixture]
    pub fn mock_subroutine_entity(
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) -> SubroutineEntity {
        SubroutineEntity::new(&mock_scene_entity.id, &mock_subroutine_image.id)
    }

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
