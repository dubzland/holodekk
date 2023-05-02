mod id;
pub use id::*;
mod scene;
pub use scene::*;
mod subroutine;
pub use subroutine::*;
mod repository;
pub use repository::*;

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use async_trait::async_trait;
    use mockall::mock;

    use super::repository::EntityRepositoryResult;
    use super::{
        MockSceneEntityRepository, MockSubroutineEntityRepository, SceneEntityRepository,
        SceneEntityRepositoryQuery, SubroutineEntityRepository, SubroutineEntityRepositoryQuery,
    };

    use crate::enums::{SceneStatus, SubroutineStatus};
    use crate::images::{fixtures::mock_subroutine_image, SubroutineImage};

    use super::*;

    #[fixture]
    pub fn mock_scene_entity() -> SceneEntity {
        let mut scene = SceneEntity::new("test".into());
        scene.created_at = Some(chrono::Utc::now().naive_utc());
        scene
    }

    #[fixture]
    pub fn mock_subroutine_entity(
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) -> SubroutineEntity {
        let mut subroutine =
            SubroutineEntity::new(&mock_scene_entity.id, &mock_subroutine_image.id);
        subroutine.created_at = Some(chrono::Utc::now().naive_utc());
        subroutine
    }

    #[fixture]
    pub fn mock_scene_entity_repository() -> MockSceneEntityRepository {
        MockSceneEntityRepository::default()
    }

    #[fixture]
    pub fn mock_subroutine_entity_repository() -> MockSubroutineEntityRepository {
        MockSubroutineEntityRepository::default()
    }

    mock! {
        pub EntityRepository {}

        #[async_trait]
        impl SceneEntityRepository for EntityRepository {
            async fn scenes_create(
                &self,
                scene: SceneEntity,
            ) -> EntityRepositoryResult<SceneEntity>;
            async fn scenes_delete(&self, id: &SceneEntityId) -> EntityRepositoryResult<()>;
            async fn scenes_exists<'a>(&self, query: SceneEntityRepositoryQuery<'a>) -> EntityRepositoryResult<bool>;
            async fn scenes_find<'a>(&self, query: SceneEntityRepositoryQuery<'a>)
                -> EntityRepositoryResult<Vec<SceneEntity>>;
            async fn scenes_get(&self, id: &SceneEntityId) -> EntityRepositoryResult<SceneEntity>;
            async fn scenes_update(&self, id: &SceneEntityId, name: Option<SceneName>, status: Option<SceneStatus>) -> EntityRepositoryResult<SceneEntity>;
        }

        #[async_trait]
        impl SubroutineEntityRepository for EntityRepository {
            async fn subroutines_create(
                &self,
                subroutine: SubroutineEntity,
            ) -> EntityRepositoryResult<SubroutineEntity>;
            async fn subroutines_delete(&self, id: &SubroutineEntityId) -> EntityRepositoryResult<()>;
            async fn subroutines_exists<'a>(&self, query: SubroutineEntityRepositoryQuery<'a>) -> EntityRepositoryResult<bool>;
            async fn subroutines_find<'a>(
                &self,
                query: SubroutineEntityRepositoryQuery<'a>,
            ) -> EntityRepositoryResult<Vec<SubroutineEntity>>;
            async fn subroutines_get(&self, id: &SubroutineEntityId) -> EntityRepositoryResult<SubroutineEntity>;
            async fn subroutines_update(&self, id: &SubroutineEntityId, status: Option<SubroutineStatus>) -> EntityRepositoryResult<SubroutineEntity>;
        }
    }

    #[fixture]
    pub(crate) fn mock_entity_repository() -> MockEntityRepository {
        MockEntityRepository::default()
    }
}
