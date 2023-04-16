use async_trait::async_trait;
use log::{debug, info, trace, warn};
#[cfg(test)]
use mockall::*;
use tokio::sync::mpsc::Sender;

use crate::core::projectors::{
    entities::Projector,
    repositories::{projector_repo_id, ProjectorsRepository},
};
use crate::core::services::{Error, Result};

use super::{ProjectorCommand, ProjectorsService};

#[derive(Clone, Debug)]
pub struct ProjectorsCreateInput<'c> {
    namespace: &'c str,
}

impl<'c> ProjectorsCreateInput<'c> {
    pub fn new(namespace: &'c str) -> Self {
        Self { namespace }
    }

    pub fn namespace(&self) -> &str {
        self.namespace
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateProjector {
    async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<Projector>;
}

#[async_trait]
impl<R> CreateProjector for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<Projector> {
        trace!("ProjectorsService.create({:?})", input);

        // ensure a projector is not already running for this namespace
        let id = projector_repo_id(&self.fleet, input.namespace());
        if self.repo.projectors_exists(&id).await? {
            warn!(
                "projector already running for namespace: {}",
                input.namespace
            );
            Err(Error::Duplicate)
        } else {
            // send spawn request to manager
            info!("Spawning projector for namespace: {}", input.namespace);
            let projector: Projector = send_start_command(self.worker(), input.namespace()).await?;
            info!("Projector spawned: {:?}", projector);

            // store the projector and return it
            let projector = self.repo.projectors_create(projector).await?;
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

    use mockall::predicate::*;
    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
    };
    use crate::core::services::Error;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_projector_already_running(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
    ) {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);
        projectors_repository
            .expect_projectors_exists()
            .return_const(Ok(true));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        let res = service
            .create(&ProjectorsCreateInput {
                namespace: "existing",
            })
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::Duplicate));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_start_command_to_manager_and_adds_record(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
        projector: Projector,
    ) {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);

        // projector does not exist
        projectors_repository
            .expect_projectors_exists()
            .return_const(Ok(false));

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
        projectors_repository
            .expect_projectors_create()
            .with(eq(projector.clone()))
            .return_const(Ok(projector.clone()));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        service
            .create(&ProjectorsCreateInput {
                namespace: "nonexistent",
            })
            .await
            .unwrap();
    }
}
