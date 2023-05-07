use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

use crate::core::subroutine::{
    entity::{Id, Repository},
    Entity,
};
use crate::entity::{
    repository::Error as RepositoryError,
    service::{Error, Result},
};

use super::Service;

/// Input requirements for [`Get::get()`]
#[derive(Clone, Debug)]
pub struct Input<'a> {
    /// Specific entity id to retrieve from the repository
    pub id: &'a str,
}

impl<'a> Input<'a> {
    /// Shorthand for instantiating a new [`Input`] struct.
    #[must_use]
    pub fn new(id: &'a str) -> Self {
        Self { id }
    }
}

/// Retrieve a given [`subroutine::Entity`][`Entity`] from the repository.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Get: Send + Sync + 'static {
    /// Retrieves the subroutine entity matching the input from the repository.
    async fn get<'a>(&self, input: &'a Input<'a>) -> Result<Entity>;
}

#[async_trait]
impl<R> Get for Service<R>
where
    R: Repository,
{
    async fn get<'a>(&self, input: &'a Input<'a>) -> Result<Entity> {
        let id: Id = input.id.parse()?;

        let subroutine = self.repo.subroutines_get(&id).await.map_err(|err| {
            if matches!(err, RepositoryError::NotFound(..)) {
                Error::NotFound(id)
            } else {
                Error::from(err)
            }
        })?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;
    use rstest::*;

    use crate::core::subroutine::entity::{
        mock_entity,
        repository::{mock_repository, MockRepository},
    };

    use super::*;

    async fn execute(repo: MockRepository, id: &str) -> Result<Entity> {
        let service = Service::new(Arc::new(repo));

        service.get(&Input::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_subroutine(mut mock_repository: MockRepository) {
        let id = Id::generate();

        {
            let sub_id = id.clone();
            mock_repository
                .expect_subroutines_get()
                .with(eq(sub_id))
                .return_once(|id| Err(RepositoryError::NotFound(id.to_owned())));
        }

        assert!(matches!(
            execute(mock_repository, &id).await.unwrap_err(),
            Error::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_for_existing_subroutine(
        mut mock_repository: MockRepository,
        mock_entity: Entity,
    ) {
        let id = mock_entity.id.clone();

        {
            let sub = mock_entity.clone();
            mock_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub.clone()));
        }

        assert_eq!(execute(mock_repository, &id).await.unwrap(), mock_entity);
    }
}
