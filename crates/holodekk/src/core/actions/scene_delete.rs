use std::sync::Arc;

use log::trace;

use crate::core::{
    entities::SceneEntityId,
    repositories::{self, ScenesRepository},
};

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub id: &'a SceneEntityId,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid scene id")]
    NotFound(SceneEntityId),
    #[error("General repository error occurred")]
    Repository(#[from] repositories::Error),
}

pub type Result = std::result::Result<(), Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: ScenesRepository,
{
    trace!("delete_scene::execute({:?})", request);

    // ensure the scene exists
    let scene = repo.scenes_get(request.id).await.map_err(|err| match err {
        repositories::Error::NotFound(id) => Error::NotFound(id),
        _ => Error::from(err),
    })?;

    // remove scene from the repository
    repo.scenes_delete(&scene.id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::{
        entities::{fixtures::mock_scene_entity, SceneEntity},
        repositories::{fixtures::mock_scenes_repository, MockScenesRepository},
    };

    use super::*;

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
            .return_once(move |id| Err(repositories::Error::NotFound(id.clone())));

        let res = execute(Arc::new(mock_scenes_repository), Request { id: &mock_id }).await;

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
        let id = mock_scene_entity.id.clone();
        let scenes_delete_result = Ok(());
        mock_scenes_repository
            .expect_scenes_delete()
            .with(eq(id))
            .return_once(move |_| scenes_delete_result);

        execute(
            Arc::new(mock_scenes_repository),
            Request {
                id: &mock_scene_entity.id,
            },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene_entity: SceneEntity,
    ) {
        let scenes_get_result = Ok(mock_scene_entity.clone());
        mock_scenes_repository
            .expect_scenes_get()
            .return_once(move |_| scenes_get_result);

        let scenes_delete_result = Ok(());
        mock_scenes_repository
            .expect_scenes_delete()
            .return_once(move |_| scenes_delete_result);

        let result = execute(
            Arc::new(mock_scenes_repository),
            Request {
                id: &mock_scene_entity.id,
            },
        )
        .await;

        assert!(result.is_ok());
    }
}
