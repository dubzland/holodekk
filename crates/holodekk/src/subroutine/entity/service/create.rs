use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

use crate::entity::service::{Error, Result};
use crate::subroutine::{
    entity::{repository::Query, Id, Repository, Status},
    image, Entity,
};

use super::Service;

#[derive(Clone, Debug)]
pub struct Input<'a> {
    pub scene_entity_id: &'a str,
    pub image_id: &'a str,
}

impl<'a> Input<'a> {
    pub fn new(scene_entity_id: &'a str, image_id: &'a str) -> Self {
        Self {
            scene_entity_id,
            image_id,
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create: Send + Sync + 'static {
    async fn create<'a>(&self, input: &'a Input<'a>) -> Result<Entity>;
}

#[async_trait]
impl<R> Create for Service<R>
where
    R: Repository,
{
    async fn create<'a>(&self, input: &'a Input<'a>) -> Result<Entity> {
        let scene_entity_id: Id = input.scene_entity_id.parse()?;
        let image_id: image::Id = input.image_id.parse()?;

        let query = Query::builder()
            .for_scene_entity(&scene_entity_id)
            .for_image(&image_id)
            .build();

        if self.repo.subroutines_exists(query).await? {
            Err(Error::NotUnique(format!(
                "Scene already exists: {scene_entity_id} - {image_id}",
            )))
        } else {
            let mut subroutine = Entity::new(&scene_entity_id, &image_id);
            subroutine.status = Status::Unknown;
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

    use crate::scene::{entity::mock_entity as mock_scene_entity, Entity as SceneEntity};
    use crate::subroutine::{
        entity::repository::{mock_repository, MockRepository},
        image::mock_image,
        Image,
    };

    use super::*;

    async fn execute(repo: MockRepository, scene: &str, image: &str) -> Result<Entity> {
        let service = Service::new(Arc::new(repo));

        service.create(&Input::new(scene, image)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_already_exists(
        mut mock_repository: MockRepository,
        mock_scene_entity: SceneEntity,
        mock_image: Image,
    ) {
        // subroutine already exists
        let scene_id = mock_scene_entity.id.clone();
        let image_id = mock_image.id.clone();
        mock_repository
            .expect_subroutines_exists()
            .withf(move |query| {
                query
                    == &Query::builder()
                        .for_scene_entity(&scene_id)
                        .for_image(&image_id)
                        .build()
            })
            .return_once(move |_| Ok(true));

        let res = execute(mock_repository, &mock_scene_entity.id, &mock_image.id).await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotUnique(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn adds_entity_to_repository(
        mut mock_repository: MockRepository,
        mock_scene_entity: SceneEntity,
        mock_image: Image,
    ) {
        let scene_id = mock_scene_entity.id.clone();
        let image_id = mock_image.id.clone();
        let status = Status::Unknown;

        mock_repository
            .expect_subroutines_exists()
            .return_once(move |_| Ok(false));

        // expect creation
        mock_repository
            .expect_subroutines_create()
            .withf(move |sub| {
                &sub.scene_entity_id == &scene_id
                    && &sub.image_id == &image_id
                    && &sub.status == &status
            })
            .return_once(move |mut sub| {
                sub.created();
                sub.updated();
                Ok(sub)
            });

        execute(mock_repository, &mock_scene_entity.id, &mock_image.id)
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_new_subroutine(
        mut mock_repository: MockRepository,
        mock_scene_entity: SceneEntity,
        mock_image: Image,
    ) {
        let scene_id = mock_scene_entity.id.clone();
        let image_id = mock_image.id.clone();
        let status = Status::Unknown;

        mock_repository
            .expect_subroutines_exists()
            .return_once(move |_| Ok(false));

        mock_repository
            .expect_subroutines_create()
            .return_once(move |mut sub| {
                sub.created();
                sub.updated();
                Ok(sub)
            });

        let new_subroutine = execute(mock_repository, &mock_scene_entity.id, &mock_image.id)
            .await
            .unwrap();
        assert_eq!(new_subroutine.scene_entity_id, scene_id);
        assert_eq!(new_subroutine.image_id, image_id);
        assert_eq!(new_subroutine.status, status);
    }
}
