use async_trait::async_trait;
use log::trace;

use crate::core::entities::{SceneEntity, SceneEntityRepository, SceneEntityRepositoryQuery};

use super::{EntityServiceResult, FindScenes, FindScenesInput, SceneEntityService};

#[async_trait]
impl<R> FindScenes for SceneEntityService<R>
where
    R: SceneEntityRepository,
{
    async fn find<'a>(
        &self,
        input: &'a FindScenesInput<'a>,
    ) -> EntityServiceResult<Vec<SceneEntity>> {
        trace!("SceneEntityService#delete({:?})", input);

        let query = SceneEntityRepositoryQuery::default();

        let scenes = self.repo.scenes_find(query).await?;
        Ok(scenes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::entities::{
        fixtures::{mock_scene_entity, mock_scene_entity_repository},
        MockSceneEntityRepository, SceneEntity,
    };

    use super::*;

    async fn execute(repo: MockSceneEntityRepository) -> EntityServiceResult<Vec<SceneEntity>> {
        let service = SceneEntityService::new(Arc::new(repo));

        service.find(&FindScenesInput::default()).await
    }

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_scene_entity_repository: MockSceneEntityRepository) {
        mock_scene_entity_repository
            .expect_scenes_find()
            .withf(|query: &SceneEntityRepositoryQuery| {
                query == &SceneEntityRepositoryQuery::default()
            })
            .return_once(move |_| Ok(vec![]));

        execute(mock_scene_entity_repository).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_scene_entity_repository: MockSceneEntityRepository,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entities = vec![mock_scene_entity.clone()];
            mock_scene_entity_repository
                .expect_scenes_find()
                .return_once(move |_| Ok(entities));
        }

        let scenes = execute(mock_scene_entity_repository).await.unwrap();
        assert_eq!(scenes, vec![mock_scene_entity]);
    }
}
