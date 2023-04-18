use async_trait::async_trait;

use crate::core::repositories::RepositoryError;

use crate::core::subroutine_definitions::SubroutineDefinitionsServiceMethods;
use crate::core::subroutines::{
    entities::Subroutine, repositories::SubroutinesRepository, worker::SubroutineCommand,
    GetSubroutine, Result, SubroutinesError, SubroutinesGetInput,
};
use crate::utils::Worker;

use super::SubroutinesService;

#[async_trait]
impl<R, W, D> GetSubroutine for SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn get<'a>(&self, input: &'a SubroutinesGetInput) -> Result<Subroutine> {
        let subroutine = self.repo.subroutines_get(input.id()).await.map_err(|err| {
            if matches!(err, RepositoryError::NotFound(..)) {
                SubroutinesError::NotFound(input.id().into())
            } else {
                SubroutinesError::from(err)
            }
        })?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::repositories::{RepositoryError, RepositoryId};
    use crate::core::subroutine_definitions::fixtures::{
        mock_subroutine_definitions_service, MockSubroutineDefinitionsService,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, Subroutine},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        worker::{fixtures::mock_subroutines_worker, MockSubroutinesWorker},
        Result, SubroutinesError,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_for_existing_subroutine(
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mock_subroutines_worker: MockSubroutinesWorker,
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: Subroutine,
    ) -> Result<()> {
        let id = subroutine.id().to_string();

        subroutines_repository
            .expect_subroutines_get()
            .withf(move |i| i == id)
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        assert_eq!(
            service
                .get(&SubroutinesGetInput::new(&subroutine.id()))
                .await?,
            subroutine
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_subroutine(
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mock_subroutines_worker: MockSubroutinesWorker,
        mut subroutines_repository: MockSubroutinesRepository,
    ) -> Result<()> {
        subroutines_repository
            .expect_subroutines_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        assert!(matches!(
            service
                .get(&SubroutinesGetInput::new("nonexistent"))
                .await
                .unwrap_err(),
            SubroutinesError::NotFound(..)
        ));
        Ok(())
    }
}
