use async_trait::async_trait;
use log::trace;

use crate::entities::{EntityRepositoryError, SceneEntityId, SceneEntityRepository};
use crate::services::{EntityServiceError, EntityServiceResult};

use super::{DeleteScene, DeleteSceneInput, SceneEntityService};

#[async_trait]
impl<R> DeleteScene for SceneEntityService<R>
where
    R: SceneEntityRepository,
{
    async fn delete<'a>(&self, input: &'a DeleteSceneInput<'a>) -> EntityServiceResult<()> {
        trace!("SceneEntityService#delete({:?}", input);

        let id: SceneEntityId = input.id.parse()?;

        // ensure the scene exists
        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            EntityRepositoryError::NotFound(id) => EntityServiceError::NotFound(id),
            _ => EntityServiceError::from(err),
        })?;

        // remove scene from the repository
        self.repo.scenes_delete(&scene.id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::entities::{
        fixtures::{mock_scene_entity, mock_scene_entity_repository},
        EntityRepositoryError, MockSceneEntityRepository, SceneEntity,
    };

    use super::*;

    async fn execute(repo: MockSceneEntityRepository, id: &str) -> EntityServiceResult<()> {
        let service = SceneEntityService::new(Arc::new(repo));

        service.delete(&DeleteSceneInput::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_scene(
        mut mock_scene_entity_repository: MockSceneEntityRepository,
    ) {
        let mock_id = SceneEntityId::generate();

        // scene does not exist
        mock_scene_entity_repository
            .expect_scenes_get()
            .with(eq(mock_id.clone()))
            .return_once(move |id| Err(EntityRepositoryError::NotFound(id.clone())));

        let res = execute(mock_scene_entity_repository, &mock_id).await;

        assert!(matches!(res.unwrap_err(), EntityServiceError::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(
        mut mock_scene_entity_repository: MockSceneEntityRepository,
        mock_scene_entity: SceneEntity,
    ) {
        // scene exists
        {
            let entity = mock_scene_entity.clone();
            mock_scene_entity_repository
                .expect_scenes_get()
                .return_once(move |_| Ok(entity));
        }

        // expect deletion
        mock_scene_entity_repository
            .expect_scenes_delete()
            .with(eq(mock_scene_entity.id.clone()))
            .return_once(move |_| Ok(()));

        execute(mock_scene_entity_repository, &mock_scene_entity.id)
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(
        mut mock_scene_entity_repository: MockSceneEntityRepository,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_scene_entity_repository
                .expect_scenes_get()
                .return_once(move |_| Ok(entity));
        }

        mock_scene_entity_repository
            .expect_scenes_delete()
            .return_once(move |_| Ok(()));

        let result = execute(mock_scene_entity_repository, &mock_scene_entity.id).await;

        assert!(result.is_ok());
    }
}
