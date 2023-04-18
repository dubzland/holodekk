use async_trait::async_trait;
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::Sender;

use crate::core::services::{Error, Result};
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, SubroutineDefinitionsGetInput,
    SubroutineDefinitionsServiceMethods,
};
use crate::core::subroutines::{
    entities::Subroutine,
    repositories::{subroutine_repo_id, SubroutinesRepository},
    CreateSubroutine, SubroutinesCreateInput,
};
use crate::utils::Worker;

use super::{SubroutineCommand, SubroutinesService};

#[async_trait]
impl<R, W, D> CreateSubroutine for SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn create<'c>(&self, input: &'c SubroutinesCreateInput<'c>) -> Result<Subroutine> {
        trace!("SubroutinesService.create({:?})", input);

        // ensure this subroutine isn't already running in the selected namespace
        let id = subroutine_repo_id(input.fleet, input.namespace, input.subroutine_definition_id);
        if self.repo.subroutines_exists(&id).await? {
            Err(Error::AlreadyRunning)
        } else {
            // retrieve the subroutine definition
            let subroutine_definition = self
                .definitions
                .get(&SubroutineDefinitionsGetInput::new(
                    input.subroutine_definition_id(),
                ))
                .await?;

            // send spawn request to manager
            info!(
                "Spawning subroutine {} in namespace {}",
                subroutine_definition.name(),
                input.namespace
            );
            let subroutine: Subroutine = send_start_command(
                self.worker(),
                input.namespace,
                subroutine_definition.clone(),
            )
            .await?;
            info!("Subroutine spawned: {:?}", subroutine);

            // store the instance and return it
            let subroutine = self.repo.subroutines_create(subroutine).await?;
            Ok(subroutine)
        }
    }
}

async fn send_start_command(
    manager: Sender<SubroutineCommand>,
    namespace: &str,
    subroutine_definition: SubroutineDefinition,
) -> Result<Subroutine> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    let cmd = SubroutineCommand::Spawn {
        namespace: namespace.to_string(),
        definition: subroutine_definition,
        resp: resp_tx,
    };

    debug!("command: {:?}", cmd);

    manager.send(cmd).await.map_err(|err| {
        warn!(
            "Failed to send subroutine spawn command to manager: {}",
            err
        );
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
