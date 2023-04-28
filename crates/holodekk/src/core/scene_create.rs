use std::sync::Arc;

use log::{trace, warn};

use crate::core::{
    entities::{SceneEntity, SceneName},
    enums::SceneStatus,
    repositories::{ScenesQuery, ScenesRepository},
};
use crate::repositories::RepositoryError;

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub name: &'a SceneName,
    pub status: &'a SceneStatus,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Scene already exists for specified name")]
    Conflict(String),
    #[error("General repository error occurred")]
    Repository(#[from] RepositoryError),
}

pub type Result = std::result::Result<SceneEntity, Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: ScenesRepository,
{
    trace!("create_scene#execute({:?})", request);

    // ensure a scene does not exist for this name
    let query = ScenesQuery::builder().name_eq(request.name).build();

    if repo.scenes_exists(query).await? {
        warn!("scene already running for name: {}", request.name);
        Err(Error::Conflict(request.name.into()))
    } else {
        let scene = repo.scenes_create(SceneEntity::from(request)).await?;
        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;
    use timestamps::Timestamps;

    use crate::core::entities::fixtures::mock_scene;
    use crate::core::repositories::{fixtures::mock_scenes_repository, MockScenesRepository};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_scene_already_exists(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene: SceneEntity,
    ) {
        let scene_name = mock_scene.name.clone();

        // scene already exists
        mock_scenes_repository
            .expect_scenes_exists()
            .withf(move |input| input.name() == Some(&scene_name))
            .return_once(move |_| Ok(true));

        let res = execute(
            Arc::new(mock_scenes_repository),
            Request {
                name: &mock_scene.name,
                status: &mock_scene.status,
            },
        )
        .await;

        assert!(matches!(res.unwrap_err(), Error::Conflict(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn adds_entity_to_repository(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene: SceneEntity,
    ) {
        // scene does not exist
        let scenes_exists_result = Ok(false);
        mock_scenes_repository
            .expect_scenes_exists()
            .return_once(move |_| scenes_exists_result);

        // expect creation
        let scene_name = mock_scene.name.clone();
        let scene_status = mock_scene.status;
        mock_scenes_repository
            .expect_scenes_create()
            .withf(move |scene| scene.name == scene_name && scene.status == scene_status)
            .return_once(move |mut scene| {
                scene.created();
                scene.updated();
                Ok(scene)
            });

        execute(
            Arc::new(mock_scenes_repository),
            Request {
                name: &mock_scene.name,
                status: &mock_scene.status,
            },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_scene_entity(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene: SceneEntity,
    ) {
        // scene does not exist
        let scenes_exists_result = Ok(false);
        mock_scenes_repository
            .expect_scenes_exists()
            .return_once(move |_| scenes_exists_result);

        // creation succeeds
        mock_scenes_repository
            .expect_scenes_create()
            .return_once(move |mut scene| {
                scene.created();
                scene.updated();
                Ok(scene)
            });

        let new_scene = execute(
            Arc::new(mock_scenes_repository),
            Request {
                name: &mock_scene.name,
                status: &mock_scene.status,
            },
        )
        .await
        .unwrap();

        assert!(new_scene.name == mock_scene.name && new_scene.status == mock_scene.status);
    }
}
