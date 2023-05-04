use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity::{
    repository::Error as RepositoryError,
    service::{Error, Result},
};
use crate::scene::{
    entity::{Id, Repository},
    Entity,
};

/// Input requirements for [`Get::get()`]
#[derive(Clone, Debug)]
pub struct Input<'g> {
    /// Specific entity id to retrieve from the repository
    pub id: &'g str,
}

impl<'g> Input<'g> {
    /// Shorthand for instantiating a new [`Input`] struct.
    #[must_use]
    pub fn new(id: &'g str) -> Self {
        Self { id }
    }
}

/// Retrieve a given [`scene::Entity`][`Entity`] from the repository.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Get: Send + Sync + 'static {
    /// Retrieves the scene entity matching the input from the repository.
    async fn get<'a>(&self, input: &'a Input<'a>) -> Result<Entity>;
}

#[async_trait]
impl<R> Get for super::Service<R>
where
    R: Repository,
{
    async fn get<'a>(&self, input: &'a Input<'a>) -> Result<Entity> {
        trace!("scene::entity::Service#get({:?}", input);

        let id: Id = input.id.parse()?;

        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            RepositoryError::NotFound(id) => Error::NotFound(id),
            _ => Error::from(err),
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
