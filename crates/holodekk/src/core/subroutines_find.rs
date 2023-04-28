use std::sync::Arc;

use log::trace;

use crate::core::{
    entities::{SceneEntityId, SubroutineDefinitionEntityId, SubroutineEntity},
    repositories::{SubroutinesQuery, SubroutinesRepository},
};
use crate::repositories::RepositoryError;

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub scene_id: Option<&'a SceneEntityId>,
    pub subroutine_definition_id: Option<&'a SubroutineDefinitionEntityId>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("General repository error occurred")]
    Repository(#[from] RepositoryError),
}

pub type Result = std::result::Result<Vec<SubroutineEntity>, Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: SubroutinesRepository,
{
    trace!("subroutines_find::execute({:?})", request);
    let query = SubroutinesQuery::from(request);
    let subroutines = repo.subroutines_find(query).await?;
    Ok(subroutines)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::{
        entities::{fixtures::mock_subroutine, SubroutineEntity},
        repositories::{fixtures::mock_subroutines_repository, MockSubroutinesRepository},
    };
    use rstest::*;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_subroutines_repository: MockSubroutinesRepository) {
        let scene_id = SceneEntityId::generate();
        let subroutine_definition_id = SubroutineDefinitionEntityId::generate();

        {
            let scene_id = scene_id.clone();
            let subroutine_definition_id = subroutine_definition_id.clone();
            mock_subroutines_repository
                .expect_subroutines_find()
                .withf(move |query: &SubroutinesQuery| {
                    query.scene_id == Some(&scene_id)
                        && query.subroutine_definition_id == Some(&subroutine_definition_id)
                })
                .return_once(|_| Ok(vec![]));
        }

        execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene_id: Some(&scene_id),
                subroutine_definition_id: Some(&subroutine_definition_id),
            },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine: SubroutineEntity,
    ) {
        let scene_id = mock_subroutine.scene_id.clone();
        let subroutine_definition_id = mock_subroutine.subroutine_definition_id.clone();

        {
            let mock_subroutine = mock_subroutine.clone();
            mock_subroutines_repository
                .expect_subroutines_find()
                .return_once(move |_| Ok(vec![mock_subroutine]));
        }

        let subroutines = execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene_id: Some(&scene_id),
                subroutine_definition_id: Some(&subroutine_definition_id),
            },
        )
        .await
        .unwrap();
        assert_eq!(subroutines, vec![mock_subroutine]);
    }
}
