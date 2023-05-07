use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::core::scene::entity::{Id, Repository};
use crate::entity::{
    repository::Error as RepositoryError,
    service::{Error, Result},
};

/// Input requirements for [`Delete::delete()`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input<'d> {
    /// Specific entity id to delete from the repository
    pub id: &'d str,
}

impl<'d> Input<'d> {
    /// Shorthand for instantiating a new [`Input`] struct.
    #[must_use]
    pub fn new(id: &'d str) -> Self {
        Self { id }
    }
}

/// Delete a given [`scene::Entity`]['Entity`] from the repository.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Delete: Send + Sync + 'static {
    /// Deletes the scene entity matching the spefied [`Id`] from the repository.
    async fn delete<'a>(&self, input: &'a Input<'a>) -> Result<()>;
}

#[async_trait]
impl<R> Delete for super::Service<R>
where
    R: Repository,
{
    async fn delete<'a>(&self, input: &'a Input<'a>) -> Result<()> {
        trace!("scene::entity::Service#delete({:?}", input);

        let id: Id = input.id.parse()?;

        // ensure the scene exists
        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            RepositoryError::NotFound(id) => Error::NotFound(id),
            _ => Error::from(err),
        })?;

        // remove scene from the repository
        self.repo.scenes_delete(&scene.id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::scene::{
        self,
        entity::{
            mock_entity,
            repository::{mock_repository, MockRepository},
        },
    };
    use crate::entity;

    use super::*;

    async fn execute(repo: MockRepository, id: &str) -> entity::service::Result<()> {
        let service = scene::entity::Service::new(Arc::new(repo));

        service.delete(&super::Input::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_scene(mut mock_repository: MockRepository) {
        let mock_id = scene::entity::Id::generate();

        // scene does not exist
        mock_repository
            .expect_scenes_get()
            .with(eq(mock_id.clone()))
            .return_once(move |id| Err(entity::repository::Error::NotFound(id.clone())));

        let res = execute(mock_repository, &mock_id).await;

        assert!(matches!(
            res.unwrap_err(),
            entity::service::Error::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(
        mut mock_repository: MockRepository,
        mock_entity: scene::Entity,
    ) {
        // scene exists
        {
            let entity = mock_entity.clone();
            mock_repository
                .expect_scenes_get()
                .return_once(move |_| Ok(entity));
        }

        // expect deletion
        mock_repository
            .expect_scenes_delete()
            .with(eq(mock_entity.id.clone()))
            .return_once(move |_| Ok(()));

        execute(mock_repository, &mock_entity.id).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(mut mock_repository: MockRepository, mock_entity: scene::Entity) {
        {
            let entity = mock_entity.clone();
            mock_repository
                .expect_scenes_get()
                .return_once(move |_| Ok(entity));
        }

        mock_repository
            .expect_scenes_delete()
            .return_once(move |_| Ok(()));

        let result = execute(mock_repository, &mock_entity.id).await;

        assert!(result.is_ok());
    }
}
