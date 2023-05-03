use async_trait::async_trait;
use log::{trace, warn};
#[cfg(test)]
use mockall::automock;

use crate::entity;
use crate::scene;

#[derive(Clone, Debug)]
pub struct Input<'c> {
    pub name: &'c str,
}

impl<'c> Input<'c> {
    pub fn new(name: &'c str) -> Self {
        Self { name }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create: Send + Sync + 'static {
    async fn create<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<scene::Entity>;
}

impl From<&Input<'_>> for scene::Entity {
    fn from(input: &Input<'_>) -> scene::Entity {
        scene::Entity::new(input.name.into())
    }
}

#[async_trait]
impl<R> Create for super::Service<R>
where
    R: scene::entity::Repository,
{
    async fn create<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<scene::Entity> {
        trace!("SceneEntityService#create({:?})", input);

        // ensure a scene does not exist for this name
        let query = scene::entity::repository::Query::builder()
            .name_eq(input.name)
            .build();

        if self.repo.scenes_exists(query).await? {
            warn!("scene already exists for name: {}", input.name);
            Err(entity::service::Error::NotUnique(input.name.into()))
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

    use crate::entity;
    use crate::scene;
    use crate::scene::entity::{
        mock_entity,
        repository::{mock_repository, MockRepository},
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_scene_already_exists(mut mock_repository: MockRepository) {
        mock_repository
            .expect_scenes_exists()
            .return_once(move |_| Ok(true));

        let service = scene::entity::Service::new(Arc::new(mock_repository));

        let res = service.create(&super::Input { name: "existing" }).await;

        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            entity::service::Error::NotUnique(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn adds_entity_to_repository(
        mut mock_repository: MockRepository,
        mock_entity: scene::Entity,
    ) {
        // projector does not exist
        mock_repository
            .expect_scenes_exists()
            .return_once(|_| Ok(false));

        // expect creation
        {
            let entity = mock_entity.clone();
            let name = entity.name.clone();
            mock_repository
                .expect_scenes_create()
                .withf(move |scene| {
                    scene.name == name
                        && scene.status == scene::entity::Status::Unknown
                        && scene.created_at.is_none()
                        && scene.updated_at.is_none()
                })
                .return_once(move |_| Ok(entity.clone()));
        }

        let service = scene::entity::Service::new(Arc::new(mock_repository));

        service
            .create(&super::Input {
                name: &mock_entity.name,
            })
            .await
            .unwrap();
    }
}
