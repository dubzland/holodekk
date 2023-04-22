use anyhow::Context;
use async_trait::async_trait;
use log::{debug, info, trace};

use crate::core::projectors::{
    entities::ProjectorEntity, repositories::ProjectorsRepository, DeleteProjector,
    ProjectorsDeleteInput, ProjectorsError, Result,
};
use crate::repositories::RepositoryError;
use crate::servers::director::{DirectorError, DirectorRequest};

use super::ProjectorsService;

#[async_trait]
impl<R> DeleteProjector for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn delete<'a>(&self, input: &'a ProjectorsDeleteInput<'a>) -> Result<()> {
        trace!("ProjectorsService.stop({:?})", input);

        // ensure a projector is actually running
        let projector = self
            .repo
            .projectors_get(input.id())
            .await
            .map_err(|err| match err {
                RepositoryError::NotFound(id) => ProjectorsError::NotFound(id),
                err => ProjectorsError::from(err),
            })?;

        // send the terminate command to the manager
        info!(
            "Shutting down projector: {}:({})",
            input.id(),
            projector.namespace(),
        );
        self.send_terminate_command(projector.clone()).await?;
        info!(
            "Projector shutdown complete: {}:({})",
            input.id(),
            projector.namespace(),
        );

        // remove projector from the repository
        self.repo.projectors_delete(projector.id()).await?;
        Ok(())
    }
}

impl<R> ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn send_terminate_command(&self, projector: ProjectorEntity) -> Result<()> {
        trace!("ProjectorsService::send_shutdown_command({:?})", projector);

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

        let request = DirectorRequest::TerminateProjector {
            projector: projector.clone(),
            resp: resp_tx,
        };

        debug!("request: {:?}", request);
        self.director()
            .send(request)
            .await
            .context("Failed to send Terminate request to Director")?;

        trace!("Terminate request sent to Director.  Awaiting response...");
        let response = resp_rx
            .await
            .context("Error receiving response to Terminate request from Director")?;

        trace!("Terminate response received from Director: {:?}", response);
        response.map_err(|err| match err {
            DirectorError::ProjectorTermination(terminate) => ProjectorsError::from(terminate),
            DirectorError::Unexpected(unexpected) => ProjectorsError::from(unexpected),
            _ => ProjectorsError::from(anyhow::anyhow!(err.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::projectors::{
        entities::{fixtures::projector, ProjectorEntity},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
        ProjectorsError,
    };
    use crate::repositories::RepositoryError;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_projector(
        mut projectors_repository: MockProjectorsRepository,
    ) {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        projectors_repository
            .expect_projectors_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);
        let res = service
            .delete(&ProjectorsDeleteInput::new("nonexistent"))
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), ProjectorsError::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_stop_command_to_manager_and_removes_record(
        mut projectors_repository: MockProjectorsRepository,
        projector: ProjectorEntity,
    ) {
        let (director_tx, mut director_rx) = tokio::sync::mpsc::channel(1);
        // projector exists
        projectors_repository
            .expect_projectors_get()
            .return_const(Ok(projector.clone()));

        // Setup fake manager
        tokio::spawn(async move {
            match director_rx.recv().await.unwrap() {
                DirectorRequest::TerminateProjector { resp, .. } => {
                    resp.send(Ok(())).unwrap();
                }
                cmd => panic!("Incorrect command received from service: {:?}", cmd),
            }
        });

        let id = projector.id().to_string();
        // expect deletion
        projectors_repository
            .expect_projectors_delete()
            .with(eq(id))
            .return_const(Ok(()));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

        service
            .delete(&ProjectorsDeleteInput::new(projector.id()))
            .await
            .unwrap();
    }
}
