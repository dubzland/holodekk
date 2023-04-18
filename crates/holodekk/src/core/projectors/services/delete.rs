use async_trait::async_trait;
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::Sender;

use crate::core::projectors::{
    entities::Projector, repositories::ProjectorsRepository, DeleteProjector,
    ProjectorsDeleteInput, ProjectorsError, Result,
};
use crate::core::repositories::RepositoryError;
use crate::utils::Worker;

use super::{ProjectorCommand, ProjectorsService};

#[async_trait]
impl<R, W> DeleteProjector for ProjectorsService<R, W>
where
    R: ProjectorsRepository,
    W: Worker<Command = ProjectorCommand>,
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

        // send the shutdown command to the manager
        info!(
            "Shutting down projector: {}:({})",
            input.id(),
            projector.namespace(),
        );
        send_shutdown_command(self.worker(), projector.clone()).await?;
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

async fn send_shutdown_command(
    sender: Sender<ProjectorCommand>,
    projector: Projector,
) -> Result<()> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let cmd = ProjectorCommand::Shutdown {
        projector: projector.clone(),
        resp: resp_tx,
    };
    debug!("command: {:?}", cmd);
    sender.send(cmd).await.map_err(|err| {
        let msg = format!(
            "Failed to send projector shutdown command to manager: {}",
            err
        );
        warn!("{}", msg);
        ProjectorsError::Shutdown(msg)
    })?;

    trace!("Command sent to manager.  awaiting response...");
    let res = resp_rx.await.map_err(|err| {
        let msg = format!(
            "Failed to receive response from manager to shutdown request: {}",
            err
        );
        warn!("{}", msg);
        ProjectorsError::Shutdown(msg)
    })?;

    trace!("Response received from manager: {:?}", res);
    match res {
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = format!(
                "Manager return error in response to shutdown request: {}",
                err
            );
            warn!("{}", msg);
            Err(ProjectorsError::Shutdown(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
        worker::{fixtures::mock_projectors_worker, MockProjectorsWorker},
        ProjectorsError,
    };
    use crate::core::repositories::RepositoryError;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_projector(
        mock_config: MockConfig,
        mock_projectors_worker: MockProjectorsWorker,
        mut projectors_repository: MockProjectorsRepository,
    ) {
        projectors_repository
            .expect_projectors_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            mock_projectors_worker,
        );
        let res = service
            .delete(&ProjectorsDeleteInput::new("nonexistent"))
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), ProjectorsError::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_stop_command_to_manager_and_removes_record(
        mock_config: MockConfig,
        mut mock_projectors_worker: MockProjectorsWorker,
        mut projectors_repository: MockProjectorsRepository,
        projector: Projector,
    ) {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);
        mock_projectors_worker.expect_sender().return_const(cmd_tx);

        // projector exists
        projectors_repository
            .expect_projectors_get()
            .return_const(Ok(projector.clone()));

        // Setup fake manager
        tokio::spawn(async move {
            match cmd_rx.recv().await.unwrap() {
                ProjectorCommand::Shutdown { resp, .. } => {
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

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            mock_projectors_worker,
        );

        service
            .delete(&ProjectorsDeleteInput::new(projector.id()))
            .await
            .unwrap();
    }
}
