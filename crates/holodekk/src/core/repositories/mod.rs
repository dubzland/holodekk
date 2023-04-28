mod scene;
pub use scene::*;
mod subroutine;
pub use subroutine::*;

#[cfg(test)]
pub(crate) mod fixtures {
    use async_trait::async_trait;
    use mockall::mock;
    use rstest::*;

    use super::{
        MockScenesRepository, MockSubroutinesRepository, ScenesQuery, ScenesRepository,
        SubroutinesQuery, SubroutinesRepository,
    };
    use crate::core::entities::{
        SceneEntity, SceneEntityId, SceneName, SubroutineEntity, SubroutineEntityId,
    };
    use crate::core::enums::{SceneStatus, SubroutineStatus};
    use crate::repositories::Result;

    #[fixture]
    pub(crate) fn mock_scenes_repository() -> MockScenesRepository {
        MockScenesRepository::default()
    }

    #[fixture]
    pub(crate) fn mock_subroutines_repository() -> MockSubroutinesRepository {
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
