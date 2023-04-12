use async_trait::async_trait;
use log::{debug, info, trace, warn};
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio::sync::mpsc::Sender;

use crate::core::{
    entities::{self, Projector},
    repositories::ProjectorRepository,
    services::{Error, Result},
};
use crate::managers::projector::ProjectorCommand;

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorStartInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Start {
    /// Starts a projector daemon
    ///
    /// # Arguments
    ///
    /// `input` - parameters for the projector (currently on `namespace')
    async fn start(&self, input: ProjectorStartInput) -> Result<Projector>;
}

#[async_trait]
impl<T> Start for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn start(&self, input: ProjectorStartInput) -> Result<Projector> {
        trace!("ProjectorsService.start({:?})", input);

        // ensure a projector is not already running for this namespace
        let id = entities::projector::generate_id(&self.fleet, &input.namespace);
        if self.repo.projector_exists(&id).await {
            warn!(
                "projector already running for namespace: {}",
                input.namespace
            );
            Err(Error::Duplicate)
        } else {
            // send spawn request to manager
            info!("Spawning projector: {}", input.namespace);
            let projector: Projector =
                send_start_command(self.manager.clone(), &input.namespace).await?;
            info!("Projector spawned: {:?}", projector);

            // store the projector and return it
            let projector = self.repo.projector_create(projector).await?;
            Ok(projector)
        }
    }
}

async fn send_start_command(
    manager: Sender<ProjectorCommand>,
    namespace: &str,
) -> Result<Projector> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    let cmd = ProjectorCommand::Spawn {
        namespace: namespace.to_string(),
        resp: resp_tx,
    };

    debug!("command: {:?}", cmd);

    manager.send(cmd).await.map_err(|err| {
        warn!("Failed to send projector spawn command to manager: {}", err);
        Error::SpawnFailed
    })?;

    trace!("Command sent to manager.  awaiting response...");
    let res = resp_rx.await.map_err(|err| {
        warn!(
            "Failed to receive response from manager to spawn request: {}",
            err
        );
        Error::SpawnFailed
    })?;

    trace!("Spawn response received from manager: {:?}", res);
    res.map_err(|err| {
        warn!(
            "Manager returned error in response to spawn request: {}",
            err
        );
        Error::SpawnFailed
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::entities::{projector::fixtures::projector, Projector};
    use crate::core::repositories::{fixtures::projector_repository, MockProjectorRepository};
    use crate::core::services::Error;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_projector_already_running(
        mock_config: MockConfig,
        mut projector_repository: MockProjectorRepository,
    ) {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projector_repository
            .expect_projector_exists()
            .return_const(true);

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projector_repository),
            cmd_tx,
        );

        let res = service
            .start(ProjectorStartInput {
                namespace: "existing".to_string(),
            })
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::Duplicate));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_start_command_to_manager_and_adds_record(
        mock_config: MockConfig,
        mut projector_repository: MockProjectorRepository,
        projector: Projector,
    ) {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);

        // projector does not exist
        projector_repository
            .expect_projector_exists()
            .return_const(false);

        // Setup fake manager
        let new_projector = projector.clone();
        tokio::spawn(async move {
            match cmd_rx.recv().await.unwrap() {
                ProjectorCommand::Spawn { resp, .. } => {
                    resp.send(Ok(new_projector)).unwrap();
                }
                cmd => panic!("Incorrect command received from service: {:?}", cmd),
            }
        });

        // expect creation
        projector_repository
            .expect_projector_create()
            .with(eq(projector.clone()))
            .return_const(Ok(projector.clone()));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projector_repository),
            cmd_tx,
        );

        service
            .start(ProjectorStartInput {
                namespace: "nonexistent".to_string(),
            })
            .await
            .unwrap();
    }
}
