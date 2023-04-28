use std::sync::Arc;

use log::trace;

use crate::core::{
    entities::SceneEntity,
    repositories::{ScenesQuery, ScenesRepository},
};
use crate::repositories::RepositoryError;

#[derive(Clone, Debug)]
pub struct Request {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("General repository error occurred")]
    Repository(#[from] RepositoryError),
}

pub type Result = std::result::Result<Vec<SceneEntity>, Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request) -> Result
where
    R: ScenesRepository,
{
    trace!("find_scenes::execute({:?})", request);

    let query = ScenesQuery::default();

    let scenes = repo.scenes_find(query).await?;
    Ok(scenes)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::{
        entities::{fixtures::mock_scene, SceneEntity},
        repositories::{fixtures::mock_scenes_repository, MockScenesRepository},
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_scenes_repository: MockScenesRepository) {
        mock_scenes_repository
            .expect_scenes_find()
            .withf(|query: &ScenesQuery| query == &ScenesQuery::default())
            .return_once(move |_| Ok(vec![]));

        execute(Arc::new(mock_scenes_repository), Request {})
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene: SceneEntity,
    ) {
        let scenes_find_result = Ok(vec![mock_scene.clone()]);
        mock_scenes_repository
            .expect_scenes_find()
            .return_once(move |_| scenes_find_result);

        let scenes = execute(Arc::new(mock_scenes_repository), Request {})
            .await
            .unwrap();
        assert_eq!(scenes, vec![mock_scene]);
    }
}
