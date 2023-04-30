use std::sync::Arc;

use log::trace;

use crate::core::{
    entities::{SceneEntityId, SubroutineEntity},
    images::SubroutineImageId,
    repositories::{self, SubroutinesQuery, SubroutinesRepository},
};

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub scene_entity_id: Option<&'a SceneEntityId>,
    pub subroutine_image_id: Option<&'a SubroutineImageId>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("General repository error occurred")]
    Repository(#[from] repositories::Error),
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
        entities::{fixtures::mock_subroutine_entity, SubroutineEntity},
        images::ImageName,
        repositories::{fixtures::mock_subroutines_repository, MockSubroutinesRepository},
    };
    use rstest::*;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_subroutines_repository: MockSubroutinesRepository) {
        let scene_entity_id = SceneEntityId::generate();
        let subroutine_image_id = SubroutineImageId::generate(&ImageName::from("test"));

        {
            let scene_entity_id = scene_entity_id.clone();
            let subroutine_image_id = subroutine_image_id.clone();
            mock_subroutines_repository
                .expect_subroutines_find()
                .withf(move |query: &SubroutinesQuery| {
                    query.scene_entity_id == Some(&scene_entity_id)
                        && query.subroutine_image_id == Some(&subroutine_image_id)
                })
                .return_once(|_| Ok(vec![]));
        }

        execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene_entity_id: Some(&scene_entity_id),
                subroutine_image_id: Some(&subroutine_image_id),
            },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        let scene_entity_id = mock_subroutine_entity.scene_entity_id.clone();
        let subroutine_image_id = mock_subroutine_entity.subroutine_image_id.clone();

        {
            let mock_subroutine_entity = mock_subroutine_entity.clone();
            mock_subroutines_repository
                .expect_subroutines_find()
                .return_once(move |_| Ok(vec![mock_subroutine_entity]));
        }

        let subroutines = execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene_entity_id: Some(&scene_entity_id),
                subroutine_image_id: Some(&subroutine_image_id),
            },
        )
        .await
        .unwrap();
        assert_eq!(subroutines, vec![mock_subroutine_entity]);
    }
}
