use async_trait::async_trait;
use log::{debug, info, trace, warn};
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio::sync::mpsc::Sender;

use crate::core::{
    entities::{Subroutine, SubroutineInstance},
    repositories::{
        subroutine_instance_repo_id, SubroutineInstancesRepository, SubroutinesRepository,
    },
    services::{Error, Result},
};
use crate::managers::subroutine::SubroutineCommand;

use super::SubroutineInstancesService;

#[derive(Clone, Debug)]
pub struct SubroutineInstancesCreateInput {
    pub fleet: String,
    pub namespace: String,
    pub subroutine_id: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create {
    async fn create(&self, input: SubroutineInstancesCreateInput) -> Result<SubroutineInstance>;
}

#[async_trait]
impl<T> Create for SubroutineInstancesService<T>
where
    T: SubroutinesRepository + SubroutineInstancesRepository,
{
    async fn create(&self, input: SubroutineInstancesCreateInput) -> Result<SubroutineInstance> {
        trace!("SubroutineInstancesService.create({:?})", input);

        // ensure this subroutine isn't already running in the selected namespace
        let id = subroutine_instance_repo_id(&input.fleet, &input.namespace, &input.subroutine_id);
        if self.repo.subroutine_instances_exists(&id).await? {
            Err(Error::AlreadyRunning)
        } else {
            // retrieve the subroutine
            let subroutine = self.repo.subroutines_get(&input.subroutine_id).await?;

            // send spawn request to manager
            info!(
                "Spawning subroutine {} in namespace {}",
                subroutine.name, input.namespace
            );
            let subroutine_instance: SubroutineInstance =
                send_start_command(self.manager.clone(), &input.namespace, subroutine.clone())
                    .await?;
            info!("Subroutine spawned: {:?}", subroutine_instance);

            // store the instance and return it
            let subroutine_instance = self
                .repo
                .subroutine_instances_create(subroutine_instance)
                .await?;
            Ok(subroutine_instance)
        }
    }
}

async fn send_start_command(
    manager: Sender<SubroutineCommand>,
    namespace: &str,
    subroutine: Subroutine,
) -> Result<SubroutineInstance> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    let cmd = SubroutineCommand::Spawn {
        namespace: namespace.to_string(),
        subroutine,
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
