use anyhow::Context;
use async_trait::async_trait;
use log::{debug, info, trace};

use crate::core::projectors::ProjectorsServiceMethods;
use crate::core::subroutine_definitions::SubroutineDefinitionsServiceMethods;
use crate::core::subroutines::{
    entities::SubroutineEntity, repositories::SubroutinesRepository, DeleteSubroutine, Result,
    SubroutinesDeleteInput, SubroutinesError,
};
use crate::repositories::RepositoryError;
use crate::servers::director::{DirectorError, DirectorRequest};

use super::SubroutinesService;

#[async_trait]
impl<R, P, D> DeleteSubroutine for SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn delete<'a>(&self, input: &'a SubroutinesDeleteInput<'a>) -> Result<()> {
        trace!("SubroutinesService.stop({:?})", input);

        // ensure a subroutine is actually running
        let subroutine = self
            .repo
            .subroutines_get(input.id())
            .await
            .map_err(|err| match err {
                RepositoryError::NotFound(id) => SubroutinesError::NotFound(id),
                err => SubroutinesError::from(err),
            })?;

        // send the shutdown command to the manager
        info!("Shutting down subroutine: {}", input.id(),);

        self.send_terminate_command(subroutine.clone()).await?;
        // send_shutdown_command(self.worker(), subroutine.clone(), definition.clone()).await?;
        info!("Subroutine shutdown complete: {}", input.id(),);

        // remove subroutine from the repository
        self.repo.subroutines_delete(subroutine.id()).await?;
        Ok(())
    }
}

impl<R, P, D> SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn send_terminate_command(&self, subroutine: SubroutineEntity) -> Result<()> {
        trace!(
            "SubroutinesService::send_terminate_command({:?})",
            subroutine
        );

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

        let request = DirectorRequest::TerminateSubroutine {
            subroutine: subroutine.clone(),
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
            DirectorError::SubroutineTermination(terminate) => SubroutinesError::from(terminate),
            DirectorError::Unexpected(unexpected) => SubroutinesError::from(unexpected),
            _ => SubroutinesError::from(anyhow::anyhow!(err.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;
    use rstest::*;

    use crate::core::projectors::fixtures::{mock_projectors_service, MockProjectorsService};
    use crate::core::subroutine_definitions::fixtures::{
        mock_subroutine_definitions_service, MockSubroutineDefinitionsService,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, SubroutineEntity},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        SubroutinesError,
    };
    use crate::repositories::RepositoryError;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_subroutine(
        mock_projectors_service: MockProjectorsService,
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
    ) {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        let subroutines_get_result = Err(RepositoryError::NotFound("".into()));
        subroutines_repository
            .expect_subroutines_get()
            .return_once(move |_| subroutines_get_result);

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        let res = service
            .delete(&SubroutinesDeleteInput::new("nonexistent"))
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), SubroutinesError::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_stop_command_to_director_and_removes_record(
        mock_projectors_service: MockProjectorsService,
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: SubroutineEntity,
    ) {
        let (director_tx, mut director_rx) = tokio::sync::mpsc::channel(1);

        let subroutines_get_result = Ok(subroutine.clone());
        subroutines_repository
            .expect_subroutines_get()
            .return_once(move |_| subroutines_get_result);

        // Setup fake director
        tokio::spawn(async move {
            match director_rx.recv().await.unwrap() {
                DirectorRequest::TerminateSubroutine { resp, .. } => {
                    resp.send(Ok(())).unwrap();
                }
                cmd => panic!("Incorrect command received from service: {:?}", cmd),
            }
        });

        let id = subroutine.id().to_string();
        // expect deletion
        subroutines_repository
            .expect_subroutines_delete()
            .with(eq(id))
            .return_const(Ok(()));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        service
            .delete(&SubroutinesDeleteInput::new(&subroutine.id()))
            .await
            .unwrap();
    }
}
