use async_trait::async_trait;
use log::trace;

use crate::core::{
    entities::{EntityRepositoryError, SceneEntity, SceneEntityId, SceneEntityRepository},
    services::{Error, Result},
};

use super::{GetScene, ScenesGetInput, ScenesService};

#[async_trait]
impl<R> GetScene for ScenesService<R>
where
    R: SceneEntityRepository,
{
    async fn get<'a>(&self, input: &'a ScenesGetInput<'a>) -> Result<SceneEntity> {
        trace!("ScenesService#get({:?}", input);

        let id: SceneEntityId = input.id.parse()?;

        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            EntityRepositoryError::NotFound(id) => Error::NotFound(id),
            _ => Error::from(err),
        })?;

        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::{
        entities::{
            fixtures::{mock_scene_entity, mock_scene_entity_repository},
            EntityRepositoryError, MockSceneEntityRepository, SceneEntity,
        },
        services::scene::Result,
    };

    use super::*;

    async fn execute(repo: MockSceneEntityRepository, id: &str) -> Result<SceneEntity> {
        let service = ScenesService::new(Arc::new(repo));

        service.get(&ScenesGetInput::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_scene(
        mut mock_scene_entity_repository: MockSceneEntityRepository,
    ) {
        let mock_id = SceneEntityId::generate();

        mock_scene_entity_repository
            .expect_scenes_get()
            .return_once(move |id| Err(EntityRepositoryError::NotFound(id.to_owned())));

        let result = execute(mock_scene_entity_repository, &mock_id.to_string()).await;

        assert!(matches!(result.unwrap_err(), Error::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_scene_for_existing_scene(
        mut mock_scene_entity_repository: MockSceneEntityRepository,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_scene_entity_repository
                .expect_scenes_get()
                .with(eq(mock_scene_entity.id.clone()))
                .return_once(move |_| Ok(entity.clone()));
        }

        let scene = execute(mock_scene_entity_repository, &mock_scene_entity.id.clone())
            .await
            .unwrap();

        assert_eq!(scene, mock_scene_entity);
    }
}
