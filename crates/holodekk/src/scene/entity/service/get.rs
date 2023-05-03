use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity;
use crate::scene;

#[derive(Clone, Debug)]
pub struct Input<'g> {
    pub id: &'g str,
}

impl<'g> Input<'g> {
    pub fn new(id: &'g str) -> Self {
        Self { id }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Get: Send + Sync + 'static {
    async fn get<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<scene::Entity>;
}

#[async_trait]
impl<R> Get for super::Service<R>
where
    R: scene::entity::Repository,
{
    async fn get<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<scene::Entity> {
        trace!("scene::entity::Service#get({:?}", input);

        let id: scene::entity::Id = input.id.parse()?;

        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            entity::repository::Error::NotFound(id) => entity::service::Error::NotFound(id),
            _ => entity::service::Error::from(err),
        })?;

        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::entity;
    use crate::scene::{
        self,
        entity::{
            mock_entity,
            repository::{mock_repository, MockRepository},
        },
    };

    use super::*;

    async fn execute(repo: MockRepository, id: &str) -> entity::service::Result<scene::Entity> {
        let service = scene::entity::Service::new(Arc::new(repo));

        service.get(&super::Input::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_scene(mut mock_repository: MockRepository) {
        let mock_id = scene::entity::Id::generate();

        mock_repository
            .expect_scenes_get()
            .return_once(move |id| Err(entity::repository::Error::NotFound(id.to_owned())));

        let result = execute(mock_repository, &mock_id.to_string()).await;

        assert!(matches!(
            result.unwrap_err(),
            entity::service::Error::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_scene_for_existing_scene(
        mut mock_repository: MockRepository,
        mock_entity: scene::Entity,
    ) {
        {
            let entity = mock_entity.clone();
            mock_repository
                .expect_scenes_get()
                .with(eq(mock_entity.id.clone()))
                .return_once(move |_| Ok(entity.clone()));
        }

        let scene = execute(mock_repository, &mock_entity.id.clone())
            .await
            .unwrap();

        assert_eq!(scene, mock_entity);
    }
}
