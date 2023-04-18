use async_trait::async_trait;

use crate::core::projectors::{
    entities::Projector, repositories::ProjectorsRepository, worker::ProjectorCommand,
    GetProjector, ProjectorsError, ProjectorsGetInput, Result,
};
use crate::core::repositories::RepositoryError;
use crate::utils::Worker;

use super::ProjectorsService;

#[async_trait]
impl<R, W> GetProjector for ProjectorsService<R, W>
where
    R: ProjectorsRepository,
    W: Worker<Command = ProjectorCommand>,
{
    async fn get<'a>(&self, input: &'a ProjectorsGetInput) -> Result<Projector> {
        let projector = self.repo.projectors_get(input.id()).await.map_err(|err| {
            if matches!(err, RepositoryError::NotFound(..)) {
                ProjectorsError::NotFound(input.id().into())
            } else {
                ProjectorsError::from(err)
            }
        })?;
        Ok(projector)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
        worker::{fixtures::mock_projectors_worker, MockProjectorsWorker},
        ProjectorsError, Result,
    };
    use crate::core::repositories::RepositoryError;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_projector_for_existing_projector(
        mock_config: MockConfig,
        mock_projectors_worker: MockProjectorsWorker,
        mut projectors_repository: MockProjectorsRepository,
        projector: Projector,
    ) -> Result<()> {
        let id = projector.id().to_string();

        projectors_repository
            .expect_projectors_get()
            .withf(move |i| i == id)
            .return_const(Ok(projector.clone()));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            mock_projectors_worker,
        );

        assert_eq!(
            service
                .get(&ProjectorsGetInput::new(projector.id()))
                .await?,
            projector
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_projector(
        mock_config: MockConfig,
        mock_projectors_worker: MockProjectorsWorker,
        mut projectors_repository: MockProjectorsRepository,
    ) -> Result<()> {
        projectors_repository
            .expect_projectors_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            mock_projectors_worker,
        );

        assert!(matches!(
            service
                .get(&ProjectorsGetInput::new("nonexistent"))
                .await
                .unwrap_err(),
            ProjectorsError::NotFound(..)
        ));
        Ok(())
    }
}
