use async_trait::async_trait;
use log::{trace, warn};

use crate::core::{
    entities::{SceneEntity, ScenesQuery, ScenesRepository},
    services::{Error, Result},
};

use super::{CreateScene, ScenesCreateInput, ScenesService};

impl From<&ScenesCreateInput<'_>> for SceneEntity {
    fn from(input: &ScenesCreateInput<'_>) -> SceneEntity {
        SceneEntity::new(input.name.into())
    }
}

#[async_trait]
impl<R> CreateScene for ScenesService<R>
where
    R: ScenesRepository,
{
    async fn create<'a>(&self, input: &'a ScenesCreateInput<'a>) -> Result<SceneEntity> {
        trace!("ScenesService#create({:?})", input);

        // ensure a scene does not exist for this name
        let query = ScenesQuery::builder().name_eq(&input.name).build();

        if self.repo.scenes_exists(query).await? {
            warn!("scene already exists for name: {}", input.name);
            Err(Error::NotUnique(input.name.into()))
        } else {
            let scene = self.repo.scenes_create(input.into()).await?;
            Ok(scene)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::{
        entities::{
            fixtures::{mock_scene_entity, mock_scenes_repository},
            MockScenesRepository, SceneEntity,
        },
        enums::SceneStatus,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_scene_already_exists(
        mut mock_scenes_repository: MockScenesRepository,
    ) {
        mock_scenes_repository
            .expect_scenes_exists()
            .return_once(move |_| Ok(true));

        let service = ScenesService::new(Arc::new(mock_scenes_repository));

        let res = service
            .create(&ScenesCreateInput { name: "existing" })
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotUnique(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn adds_entity_to_repository(
        mut mock_scenes_repository: MockScenesRepository,
        mock_scene_entity: SceneEntity,
    ) {
        // projector does not exist
        mock_scenes_repository
            .expect_scenes_exists()
            .return_once(|_| Ok(false));

        // expect creation
        {
            let entity = mock_scene_entity.clone();
            let name = entity.name.clone();
            mock_scenes_repository
                .expect_scenes_create()
                .withf(move |scene| {
                    scene.name == name
                        && scene.status == SceneStatus::Unknown
                        && scene.created_at.is_none()
                        && scene.updated_at.is_none()
                })
                .return_once(move |_| Ok(entity.clone()));
        }

        let service = ScenesService::new(Arc::new(mock_scenes_repository));

        service
            .create(&ScenesCreateInput {
                name: &mock_scene_entity.name,
            })
            .await
            .unwrap();
    }
}
