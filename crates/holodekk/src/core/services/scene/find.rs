use async_trait::async_trait;
use log::trace;

use crate::core::entities::{SceneEntity, SceneEntityRepository, SceneEntityRepositoryQuery};

use super::{FindScenes, Result, ScenesFindInput, ScenesService};

#[async_trait]
impl<R> FindScenes for ScenesService<R>
where
    R: SceneEntityRepository,
{
    async fn find<'a>(&self, input: &'a ScenesFindInput<'a>) -> Result<Vec<SceneEntity>> {
        trace!("ScenesService#delete({:?})", input);

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

    async fn execute(repo: MockSceneEntityRepository) -> Result<Vec<SceneEntity>> {
        let service = ScenesService::new(Arc::new(repo));

        service.find(&ScenesFindInput::default()).await
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
