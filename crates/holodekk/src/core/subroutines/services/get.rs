use async_trait::async_trait;

use crate::repositories::RepositoryError;

use crate::core::projectors::ProjectorsServiceMethods;
use crate::core::subroutine_definitions::SubroutineDefinitionsServiceMethods;
use crate::core::subroutines::{
    entities::SubroutineEntity, repositories::SubroutinesRepository, GetSubroutine, Result,
    SubroutinesError, SubroutinesGetInput,
};

use super::SubroutinesService;

#[async_trait]
impl<R, P, D> GetSubroutine for SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn get<'a>(&self, input: &'a SubroutinesGetInput) -> Result<SubroutineEntity> {
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

    use crate::core::projectors::fixtures::{mock_projectors_service, MockProjectorsService};
    use crate::core::subroutine_definitions::fixtures::{
        mock_subroutine_definitions_service, MockSubroutineDefinitionsService,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, SubroutineEntity},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        Result, SubroutinesError,
    };
    use crate::repositories::RepositoryError;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_for_existing_subroutine(
        mock_projectors_service: MockProjectorsService,
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        let id = subroutine.id().to_string();

        subroutines_repository
            .expect_subroutines_get()
            .withf(move |i| i == id)
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
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
        mock_projectors_service: MockProjectorsService,
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        subroutines_repository
            .expect_subroutines_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
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
