use async_trait::async_trait;

use crate::core::projectors::{
    entities::ProjectorEntity, repositories::ProjectorsRepository, GetProjector, ProjectorsError,
    ProjectorsGetInput, Result,
};
use crate::repositories::RepositoryError;

use super::ProjectorsService;

#[async_trait]
impl<R> GetProjector for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn get<'a>(&self, input: &'a ProjectorsGetInput) -> Result<ProjectorEntity> {
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

    use crate::core::projectors::{
        entities::{fixtures::projector, ProjectorEntity},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
        ProjectorsError, Result,
    };
    use crate::repositories::RepositoryError;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_projector_for_existing_projector(
        mut projectors_repository: MockProjectorsRepository,
        projector: ProjectorEntity,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        let id = projector.id().to_string();

        projectors_repository
            .expect_projectors_get()
            .withf(move |i| i == id)
            .return_const(Ok(projector.clone()));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

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
        mut projectors_repository: MockProjectorsRepository,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        projectors_repository
            .expect_projectors_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

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
