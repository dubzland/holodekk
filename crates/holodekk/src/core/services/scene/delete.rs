use async_trait::async_trait;
use log::trace;

use crate::core::{
    entities::{repository, SceneEntityId, ScenesRepository},
    services::{Error, Result},
};

use super::{DeleteScene, ScenesDeleteInput, ScenesService};

#[async_trait]
impl<R> DeleteScene for ScenesService<R>
where
    R: ScenesRepository,
{
    async fn delete<'a>(&self, input: &'a ScenesDeleteInput<'a>) -> Result<()> {
        trace!("ScenesService#delete({:?}", input);

        let id: SceneEntityId = input.id.parse()?;

        // ensure the scene exists
        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            repository::Error::NotFound(id) => Error::NotFound(id),
            _ => Error::from(err),
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

    use crate::core::{
        entities::{
            fixtures::{mock_scene_entity, mock_scenes_repository},
            repository, MockScenesRepository, SceneEntity,
        },
        services::scene::Result,
    };

    use super::*;

    async fn execute(repo: MockScenesRepository, id: &str) -> Result<()> {
        let service = ScenesService::new(Arc::new(repo));

        service.delete(&ScenesDeleteInput::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_scene(
        mut mock_scenes_repository: MockScenesRepository,
    ) {
        let mock_id = SceneEntityId::generate();

        // scene does not exist
        mock_scenes_repository
            .expect_scenes_get()
            .with(eq(mock_id.clone()))
            .return_once(move |id| Err(repository::Error::NotFound(id.clone())));

        let res = execute(mock_scenes_repository, &mock_id).await;

        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene_entity: SceneEntity,
    ) {
        // scene exists
        let scenes_get_result = Ok(mock_scene_entity.clone());
        mock_scenes_repository
            .expect_scenes_get()
            .return_once(move |_| scenes_get_result);

        // expect deletion
        mock_scenes_repository
            .expect_scenes_delete()
            .with(eq(mock_scene_entity.id.clone()))
            .return_once(move |_| Ok(()));

        execute(mock_scenes_repository, &mock_scene_entity.id)
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_scenes_repository
                .expect_scenes_get()
                .return_once(move |_| Ok(entity));
        }

        mock_scenes_repository
            .expect_scenes_delete()
            .return_once(move |_| Ok(()));

        let result = execute(mock_scenes_repository, &mock_scene_entity.id).await;

        assert!(result.is_ok());
    }
}
