use async_trait::async_trait;

use crate::core::{
    entities::{repository, SubroutineEntity, SubroutineEntityId, SubroutinesRepository},
    services::{Error, Result},
};

use super::{GetSubroutine, SubroutinesGetInput, SubroutinesService};

#[async_trait]
impl<R> GetSubroutine for SubroutinesService<R>
where
    R: SubroutinesRepository,
{
    async fn get<'a>(&self, input: &'a SubroutinesGetInput<'a>) -> Result<SubroutineEntity> {
        let id: SubroutineEntityId = input.id.parse()?;

        let subroutine = self.repo.subroutines_get(&id).await.map_err(|err| {
            if matches!(err, repository::Error::NotFound(..)) {
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

    use crate::core::entities::{
        fixtures::{mock_subroutine_entity, mock_subroutines_repository},
        repository, MockSubroutinesRepository,
    };

    use super::*;

    async fn execute(repo: MockSubroutinesRepository, id: &str) -> Result<SubroutineEntity> {
        let service = SubroutinesService::new(Arc::new(repo));

        service.get(&SubroutinesGetInput::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
    ) {
        let id = SubroutineEntityId::generate();

        {
            let sub_id = id.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .with(eq(sub_id))
                .return_once(|id| Err(repository::Error::NotFound(id.to_owned())));
        }

        assert!(matches!(
            execute(mock_subroutines_repository, &id).await.unwrap_err(),
            Error::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_for_existing_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        let id = mock_subroutine_entity.id.clone();

        {
            let sub = mock_subroutine_entity.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub.clone()));
        }

        assert_eq!(
            execute(mock_subroutines_repository, &id).await.unwrap(),
            mock_subroutine_entity
        );
    }
}
