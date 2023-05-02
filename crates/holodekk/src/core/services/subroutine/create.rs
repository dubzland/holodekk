use async_trait::async_trait;

use crate::core::{
    entities::{SceneEntityId, SubroutineEntity, SubroutinesQuery, SubroutinesRepository},
    enums::SubroutineStatus,
    images::SubroutineImageId,
    services::{Error, Result},
};

use super::{CreateSubroutine, SubroutinesCreateInput, SubroutinesService};

#[async_trait]
impl<R> CreateSubroutine for SubroutinesService<R>
where
    R: SubroutinesRepository,
{
    async fn create<'a>(&self, input: &'a SubroutinesCreateInput<'a>) -> Result<SubroutineEntity> {
        let scene_entity_id: SceneEntityId = input.scene_entity_id.parse()?;
        let subroutine_image_id: SubroutineImageId = input.subroutine_image_id.parse()?;

        let query = SubroutinesQuery::builder()
            .for_scene_entity(&scene_entity_id)
            .for_subroutine_image(&subroutine_image_id)
            .build();

        if self.repo.subroutines_exists(query).await? {
            Err(Error::NotUnique(format!(
                "Scene already exists: {} - {}",
                scene_entity_id, subroutine_image_id
            )))
        } else {
            let mut subroutine = SubroutineEntity::new(&scene_entity_id, &subroutine_image_id);
            subroutine.status = SubroutineStatus::Unknown;
            let subroutine = self.repo.subroutines_create(subroutine).await?;
            Ok(subroutine)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;
    use timestamps::Timestamps;

    use crate::core::{
        entities::{
            fixtures::{mock_scene_entity, mock_subroutines_repository},
            MockSubroutinesRepository, SceneEntity, SubroutinesQuery,
        },
        images::{fixtures::mock_subroutine_image, SubroutineImage},
    };

    use super::*;

    async fn execute(
        repo: MockSubroutinesRepository,
        scene: &str,
        image: &str,
    ) -> Result<SubroutineEntity> {
        let service = SubroutinesService::new(Arc::new(repo));

        service
            .create(&SubroutinesCreateInput::new(scene, image))
            .await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_already_exists(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) {
        // subroutine already exists
        let scene_id = mock_scene_entity.id.clone();
        let definition_id = mock_subroutine_image.id.clone();
        mock_subroutines_repository
            .expect_subroutines_exists()
            .withf(move |query| {
                query
                    == &SubroutinesQuery::builder()
                        .for_scene_entity(&scene_id)
                        .for_subroutine_image(&definition_id)
                        .build()
            })
            .return_once(move |_| Ok(true));

        let res = execute(
            mock_subroutines_repository,
            &mock_scene_entity.id,
            &mock_subroutine_image.id,
        )
        .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotUnique(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn adds_entity_to_repository(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) {
        let scene_id = mock_scene_entity.id.clone();
        let definition_id = mock_subroutine_image.id.clone();
        let status = SubroutineStatus::Unknown;

        mock_subroutines_repository
            .expect_subroutines_exists()
            .return_once(move |_| Ok(false));

        // expect creation
        mock_subroutines_repository
            .expect_subroutines_create()
            .withf(move |sub| {
                &sub.scene_entity_id == &scene_id
                    && &sub.subroutine_image_id == &definition_id
                    && &sub.status == &status
            })
            .return_once(move |mut sub| {
                sub.created();
                sub.updated();
                Ok(sub)
            });

        execute(
            mock_subroutines_repository,
            &mock_scene_entity.id,
            &mock_subroutine_image.id,
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_new_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) {
        let scene_id = mock_scene_entity.id.clone();
        let image_id = mock_subroutine_image.id.clone();
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
            mock_subroutines_repository,
            &mock_scene_entity.id,
            &mock_subroutine_image.id,
        )
        .await
        .unwrap();
        assert_eq!(new_subroutine.scene_entity_id, scene_id);
        assert_eq!(new_subroutine.subroutine_image_id, image_id);
        assert_eq!(new_subroutine.status, status);
    }
}
