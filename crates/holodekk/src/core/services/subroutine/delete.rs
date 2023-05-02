use async_trait::async_trait;
use log::trace;

use crate::core::{
    entities::{repository, SubroutineEntityId, SubroutinesRepository},
    services::{Error, Result},
};

use super::{DeleteSubroutine, SubroutinesDeleteInput, SubroutinesService};

#[async_trait]
impl<R> DeleteSubroutine for SubroutinesService<R>
where
    R: SubroutinesRepository,
{
    async fn delete<'a>(&self, input: &'a SubroutinesDeleteInput<'a>) -> Result<()>
    where
        R: SubroutinesRepository,
    {
        trace!("SubroutinesService#delete({:?})", input);

        let id: SubroutineEntityId = input.id.parse()?;

        // ensure the subroutine exists
        let subroutine = self
            .repo
            .subroutines_get(&id)
            .await
            .map_err(|err| match err {
                repository::Error::NotFound(id) => Error::NotFound(id),
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

    use crate::core::entities::{
        fixtures::{mock_subroutine_entity, mock_subroutines_repository},
        repository, MockSubroutinesRepository, SubroutineEntity,
    };

    use super::*;

    async fn execute(repo: MockSubroutinesRepository, id: &str) -> Result<()> {
        let service = SubroutinesService::new(Arc::new(repo));

        service.delete(&SubroutinesDeleteInput::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
    ) {
        let mock_id = SubroutineEntityId::generate();

        // subroutine does not exist
        mock_subroutines_repository
            .expect_subroutines_get()
            .with(eq(mock_id.clone()))
            .return_once(|id| Err(repository::Error::NotFound(id.to_owned())));

        let res = execute(mock_subroutines_repository, &mock_id).await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let sub = mock_subroutine_entity.clone();
            let sub_id = sub.id.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .withf(move |id| id == &sub_id)
                .return_once(move |_| Ok(sub));
        }

        {
            let sub_id = mock_subroutine_entity.id.clone();
            mock_subroutines_repository
                .expect_subroutines_delete()
                .withf(move |id| id == &sub_id)
                .return_once(|_| Ok(()));
        }

        execute(mock_subroutines_repository, &mock_subroutine_entity.id)
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let sub = mock_subroutine_entity.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub));
        }

        mock_subroutines_repository
            .expect_subroutines_delete()
            .return_once(|_| Ok(()));

        let res = execute(mock_subroutines_repository, &mock_subroutine_entity.id).await;

        assert!(res.is_ok());
    }
}
