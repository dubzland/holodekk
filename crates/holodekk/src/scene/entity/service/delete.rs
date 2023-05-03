use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity;
use crate::scene;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input<'d> {
    pub id: &'d str,
}

impl<'d> Input<'d> {
    pub fn new(id: &'d str) -> Self {
        Self { id }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Delete: Send + Sync + 'static {
    async fn delete<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<()>;
}

#[async_trait]
impl<R> Delete for super::Service<R>
where
    R: scene::entity::Repository,
{
    async fn delete<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<()> {
        trace!("scene::entity::Service#delete({:?}", input);

        let id: scene::entity::Id = input.id.parse()?;

        // ensure the scene exists
        let scene = self.repo.scenes_get(&id).await.map_err(|err| match err {
            entity::repository::Error::NotFound(id) => entity::service::Error::NotFound(id),
            _ => entity::service::Error::from(err),
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

    use crate::entity;
    use crate::scene;
    use crate::scene::entity::{
        mock_entity,
        repository::{mock_repository, MockRepository},
    };

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
