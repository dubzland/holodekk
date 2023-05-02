use async_trait::async_trait;

use crate::core::{
    entities::{
        EntityRepositoryError, SubroutineEntity, SubroutineEntityId, SubroutineEntityRepository,
    },
    services::{EntityServiceError, EntityServiceResult},
};

use super::{GetSubroutine, GetSubroutineInput, SubroutineEntityService};

#[async_trait]
impl<R> GetSubroutine for SubroutineEntityService<R>
where
    R: SubroutineEntityRepository,
{
    async fn get<'a>(
        &self,
        input: &'a GetSubroutineInput<'a>,
    ) -> EntityServiceResult<SubroutineEntity> {
        let id: SubroutineEntityId = input.id.parse()?;

        let subroutine = self.repo.subroutines_get(&id).await.map_err(|err| {
            if matches!(err, EntityRepositoryError::NotFound(..)) {
                EntityServiceError::NotFound(id)
            } else {
                EntityServiceError::from(err)
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
        fixtures::{mock_subroutine_entity, mock_subroutine_entity_repository},
        EntityRepositoryError, MockSubroutineEntityRepository,
    };

    use super::*;

    async fn execute(
        repo: MockSubroutineEntityRepository,
        id: &str,
    ) -> EntityServiceResult<SubroutineEntity> {
        let service = SubroutineEntityService::new(Arc::new(repo));

        service.get(&GetSubroutineInput::new(id)).await
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_subroutine(
        mut mock_subroutine_entity_repository: MockSubroutineEntityRepository,
    ) {
        let id = SubroutineEntityId::generate();

        {
            let sub_id = id.clone();
            mock_subroutine_entity_repository
                .expect_subroutines_get()
                .with(eq(sub_id))
                .return_once(|id| Err(EntityRepositoryError::NotFound(id.to_owned())));
        }

        assert!(matches!(
            execute(mock_subroutine_entity_repository, &id)
                .await
                .unwrap_err(),
            EntityServiceError::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_for_existing_subroutine(
        mut mock_subroutine_entity_repository: MockSubroutineEntityRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        let id = mock_subroutine_entity.id.clone();

        {
            let sub = mock_subroutine_entity.clone();
            mock_subroutine_entity_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub.clone()));
        }

        assert_eq!(
            execute(mock_subroutine_entity_repository, &id)
                .await
                .unwrap(),
            mock_subroutine_entity
        );
    }
}
