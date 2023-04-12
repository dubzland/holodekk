use async_trait::async_trait;
use log::{debug, info, trace, warn};
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio::sync::mpsc::Sender;

use crate::core::{
    entities::{self, Projector},
    repositories::{self, ProjectorRepository},
    services::{Error, Result},
};
use crate::managers::projector::ProjectorCommand;

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorStopInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Stop {
    /// Stops a running projector
    ///
    /// # Arguments
    ///
    /// `input` - parameters for the projector (currently only `namespace`)
    async fn stop(&self, input: ProjectorStopInput) -> Result<()>;
}

#[async_trait]
impl<T> Stop for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn stop(&self, input: ProjectorStopInput) -> Result<()> {
        trace!("ProjectorsService.stop({:?})", input);

        // ensure a projector is actually running
        let id = entities::projector::generate_id(&self.fleet, &input.namespace);
        let projector = self
            .repo
            .projector_get(&id)
            .await
            .map_err(|err| match err {
                repositories::Error::NotFound => Error::NotFound,
                err => Error::from(err),
            })?;

        // send the shutdown command to the manager
        info!("Shutting down projector: {}", input.namespace);
        send_shutdown_command(self.manager.clone(), projector.clone()).await?;
        info!("Projector shutdown complete: {}", input.namespace);

        // remove projector from the repository
        self.repo.projector_delete(&projector.id).await?;
        Ok(())
    }
}

async fn send_shutdown_command(
    manager: Sender<ProjectorCommand>,
    projector: Projector,
) -> Result<()> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let cmd = ProjectorCommand::Shutdown {
        projector: projector.clone(),
        resp: resp_tx,
    };
    debug!("command: {:?}", cmd);
    manager.send(cmd).await.map_err(|err| {
        warn!(
            "Failed to send projector shutdown command to manager: {}",
            err
        );
        Error::ShutdownFailed
    })?;

    trace!("Command sent to manager.  awaiting response...");
    let res = resp_rx.await.map_err(|err| {
        warn!(
            "Failed to receive response from manager to shutdown request: {}",
            err
        );
        Error::ShutdownFailed
    })?;

    trace!("Response received from manager: {:?}", res);
    match res {
        Ok(_) => Ok(()),
        Err(err) => {
            warn!(
                "Manager return error in response to shutdown request: {}",
                err
            );
            Err(Error::ShutdownFailed)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::entities::{projector::fixtures::projector, Projector};
    use crate::core::repositories::{
        self, fixtures::projector_repository, MockProjectorRepository,
    };
    use crate::core::services::Error;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_projector(
        mock_config: MockConfig,
        mut projector_repository: MockProjectorRepository,
    ) {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projector_repository
            .expect_projector_get()
            .return_const(Err(repositories::Error::NotFound));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projector_repository),
            cmd_tx,
        );
        let res = service
            .stop(ProjectorStopInput {
                namespace: "nonexistent".to_string(),
            })
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_stop_command_to_manager(
        mock_config: MockConfig,
        mut projector_repository: MockProjectorRepository,
        projector: Projector,
    ) {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);

        // projector exists
        projector_repository
            .expect_projector_get()
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

        // expect deletion
        projector_repository
            .expect_projector_delete()
            .with(eq(projector.id))
            .return_const(Ok(()));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projector_repository),
            cmd_tx,
        );

        service
            .stop(ProjectorStopInput {
                namespace: "nonexistent".to_string(),
            })
            .await
            .unwrap();
    }
}
