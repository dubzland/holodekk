use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity::{
    repository::Error as RepositoryError,
    service::{Error, Result},
};
use crate::subroutine::entity::{Id, Repository};

use super::Service;

#[derive(Clone, Debug)]
pub struct Input<'a> {
    pub id: &'a str,
}

impl<'a> Input<'a> {
    pub fn new(id: &'a str) -> Self {
        Self { id }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Delete: Send + Sync + 'static {
    async fn delete<'a>(&self, input: &'a Input<'a>) -> Result<()>;
}

#[async_trait]
impl<R> Delete for Service<R>
where
    R: Repository,
{
    async fn delete<'a>(&self, input: &'a Input<'a>) -> Result<()>
    where
        R: Repository,
    {
        trace!("subroutine::entity::Service#delete({:?})", input);

        let id: Id = input.id.parse()?;

        // ensure the subroutine exists
        let subroutine = self
            .repo
            .subroutines_get(&id)
            .await
            .map_err(|err| match err {
                RepositoryError::NotFound(id) => Error::NotFound(id),
                _ => Error::from(err),
            })?;

        // remove subroutine from the repository
        self.repo.subroutines_delete(&subroutine.id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;
    use rstest::*;

    use crate::subroutine::{
        entity::{
            mock_entity,
            repository::{mock_repository, MockRepository},
        },
        Entity,
    };

    use super::*;

    async fn execute(repo: MockRepository, id: &str) -> Result<()> {
        let service = Service::new(Arc::new(repo));

        service.delete(&Input::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_subroutine(mut mock_repository: MockRepository) {
        let mock_id = Id::generate();

        // subroutine does not exist
        mock_repository
            .expect_subroutines_get()
            .with(eq(mock_id.clone()))
            .return_once(|id| Err(RepositoryError::NotFound(id.to_owned())));

        let res = execute(mock_repository, &mock_id).await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(mut mock_repository: MockRepository, mock_entity: Entity) {
        {
            let sub = mock_entity.clone();
            let sub_id = sub.id.clone();
            mock_repository
                .expect_subroutines_get()
                .withf(move |id| id == &sub_id)
                .return_once(move |_| Ok(sub));
        }

        {
            let sub_id = mock_entity.id.clone();
            mock_repository
                .expect_subroutines_delete()
                .withf(move |id| id == &sub_id)
                .return_once(|_| Ok(()));
        }

        execute(mock_repository, &mock_entity.id).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(mut mock_repository: MockRepository, mock_entity: Entity) {
        {
            let sub = mock_entity.clone();
            mock_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub));
        }

        mock_repository
            .expect_subroutines_delete()
            .return_once(|_| Ok(()));

        let res = execute(mock_repository, &mock_entity.id).await;

        assert!(res.is_ok());
    }
}
