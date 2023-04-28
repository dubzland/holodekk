use std::sync::Arc;

use log::trace;

use crate::core::entities::SceneEntityId;
use crate::core::repositories::ScenesRepository;
use crate::repositories::RepositoryError;

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub id: &'a SceneEntityId,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid scene id")]
    NotFound(SceneEntityId),
    #[error("General repository error occurred")]
    Repository(#[from] RepositoryError),
}

pub type Result = std::result::Result<(), Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: ScenesRepository,
{
    trace!("delete_scene::execute({:?})", request);

    // ensure the scene exists
    let scene = repo.scenes_get(request.id).await.map_err(|err| match err {
        RepositoryError::NotFound(id) => Error::NotFound(id),
        _ => Error::from(err),
    })?;

    // remove scene from the repository
    repo.scenes_delete(&scene.id).await?;

    Ok(())
}

// impl<R> ScenesService<R>
// where
//     R: ScenesRepository,
// {
//     async fn send_terminate_command(&self, scene: SceneEntity) -> Result<()> {
//         trace!("ScenesService::send_shutdown_command({:?})", scene);

//         let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

//         let request = DirectorRequest::TerminateScene {
//             scene: scene.clone(),
//             resp: resp_tx,
//         };

//         debug!("request: {:?}", request);
//         self.director()
//             .send(request)
//             .await
//             .context("Failed to send Terminate request to Director")?;

//         trace!("Terminate request sent to Director.  Awaiting response...");
//         let response = resp_rx
//             .await
//             .context("Error receiving response to Terminate request from Director")?;

//         trace!("Terminate response received from Director: {:?}", response);
//         response.map_err(|err| match err {
//             DirectorError::SceneTermination(terminate) => ScenesError::from(terminate),
//             DirectorError::Unexpected(unexpected) => ScenesError::from(unexpected),
//             _ => ScenesError::from(anyhow::anyhow!(err.to_string())),
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::{
        entities::{fixtures::mock_scene, SceneEntity},
        repositories::{fixtures::mock_scenes_repository, MockScenesRepository},
    };
    use crate::repositories::RepositoryError;

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
            .return_once(move |id| Err(RepositoryError::NotFound(id.clone())));

        let res = execute(Arc::new(mock_scenes_repository), Request { id: &mock_id }).await;

        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene: SceneEntity,
    ) {
        // scene exists
        let scenes_get_result = Ok(mock_scene.clone());
        mock_scenes_repository
            .expect_scenes_get()
            .return_once(move |_| scenes_get_result);

        // expect deletion
        let id = mock_scene.id.clone();
        let scenes_delete_result = Ok(());
        mock_scenes_repository
            .expect_scenes_delete()
            .with(eq(id))
            .return_once(move |_| scenes_delete_result);

        execute(
            Arc::new(mock_scenes_repository),
            Request { id: &mock_scene.id },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(mut mock_scenes_repository: MockScenesRepository, mock_scene: SceneEntity) {
        let scenes_get_result = Ok(mock_scene.clone());
        mock_scenes_repository
            .expect_scenes_get()
            .return_once(move |_| scenes_get_result);

        let scenes_delete_result = Ok(());
        mock_scenes_repository
            .expect_scenes_delete()
            .return_once(move |_| scenes_delete_result);

        let result = execute(
            Arc::new(mock_scenes_repository),
            Request { id: &mock_scene.id },
        )
        .await;

        assert!(result.is_ok());
    }
}
