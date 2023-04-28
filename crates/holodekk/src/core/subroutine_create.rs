use std::sync::Arc;

use crate::core::{
    entities::{
        SceneEntity, SceneEntityId, SubroutineDefinitionEntity, SubroutineDefinitionEntityId,
        SubroutineEntity,
    },
    enums::SubroutineStatus,
    repositories::{SubroutinesQuery, SubroutinesRepository},
};

use crate::repositories::RepositoryError;

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub scene: &'a SceneEntity,
    pub subroutine_definition: &'a SubroutineDefinitionEntity,
    pub status: SubroutineStatus,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Subroutine already exists for definition {0} on scene {1}")]
    Conflict(SubroutineDefinitionEntityId, SceneEntityId),
    #[error("General repository error occurred")]
    Repository(#[from] RepositoryError),
}

pub type Result = std::result::Result<SubroutineEntity, Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: SubroutinesRepository,
{
    let query = SubroutinesQuery::builder()
        .for_subroutine_definition(&request.subroutine_definition.id)
        .for_scene(&request.scene.id)
        .build();

    if repo.subroutines_exists(query).await? {
        Err(Error::Conflict(
            request.subroutine_definition.id.clone(),
            request.scene.id.to_owned(),
        ))
    } else {
        let mut subroutine =
            SubroutineEntity::new(&request.scene.id, &request.subroutine_definition.id);
        subroutine.status = request.status;
        let subroutine = repo.subroutines_create(subroutine).await?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;
    use timestamps::Timestamps;

    use crate::core::entities::fixtures::{mock_scene, mock_subroutine_definition};
    use crate::core::repositories::{
        fixtures::mock_subroutines_repository, MockSubroutinesRepository, SubroutinesQuery,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_already_exists(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_scene: SceneEntity,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        // subroutine already exists
        let scene_id = mock_scene.id.clone();
        let definition_id = mock_subroutine_definition.id.clone();
        mock_subroutines_repository
            .expect_subroutines_exists()
            .withf(move |query| {
                query
                    == &SubroutinesQuery::builder()
                        .for_scene(&scene_id)
                        .for_subroutine_definition(&definition_id)
                        .build()
            })
            .return_once(move |_| Ok(true));

        let res = execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene: &mock_scene,
                subroutine_definition: &mock_subroutine_definition,
                status: SubroutineStatus::Unknown,
            },
        )
        .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::Conflict(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn adds_entity_to_repository(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_scene: SceneEntity,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let scene_id = mock_scene.id.clone();
        let definition_id = mock_subroutine_definition.id.clone();
        let status = SubroutineStatus::Unknown;

        mock_subroutines_repository
            .expect_subroutines_exists()
            .return_once(move |_| Ok(false));

        // expect creation
        mock_subroutines_repository
            .expect_subroutines_create()
            .withf(move |sub| {
                &sub.scene_id == &scene_id
                    && &sub.subroutine_definition_id == &definition_id
                    && &sub.status == &status
            })
            .return_once(move |mut sub| {
                sub.created();
                sub.updated();
                Ok(sub)
            });

        execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene: &mock_scene,
                subroutine_definition: &mock_subroutine_definition,
                status,
            },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_new_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_scene: SceneEntity,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let scene_id = mock_scene.id.clone();
        let definition_id = mock_subroutine_definition.id.clone();
        let status = SubroutineStatus::Unknown;

        mock_subroutines_repository
            .expect_subroutines_exists()
            .return_once(move |_| Ok(false));

        mock_subroutines_repository
            .expect_subroutines_create()
            .return_once(move |mut sub| {
                sub.created();
                sub.updated();
                Ok(sub)
            });

        let new_subroutine = execute(
            Arc::new(mock_subroutines_repository),
            Request {
                scene: &mock_scene,
                subroutine_definition: &mock_subroutine_definition,
                status,
            },
        )
        .await
        .unwrap();
        assert_eq!(new_subroutine.scene_id, scene_id);
        assert_eq!(new_subroutine.subroutine_definition_id, definition_id);
        assert_eq!(new_subroutine.status, status);
    }
}
