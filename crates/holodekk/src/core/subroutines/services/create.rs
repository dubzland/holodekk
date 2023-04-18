use async_trait::async_trait;
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::Sender;

use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, SubroutineDefinitionsError, SubroutineDefinitionsGetInput,
    SubroutineDefinitionsServiceMethods,
};
use crate::core::subroutines::{
    entities::Subroutine,
    repositories::{subroutine_repo_id, SubroutinesRepository},
    CreateSubroutine, Result, SubroutinesCreateInput, SubroutinesError,
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
            Err(SubroutinesError::AlreadyRunning(id))
        } else {
            // retrieve the subroutine definition
            match self
                .definitions
                .get(&SubroutineDefinitionsGetInput::new(
                    input.subroutine_definition_id(),
                ))
                .await
            {
                Ok(definition) => {
                    // send spawn request to manager
                    info!(
                        "Spawning subroutine {} in namespace {}",
                        definition.name(),
                        input.namespace
                    );
                    let subroutine: Subroutine =
                        send_start_command(self.worker(), input.namespace, definition.clone())
                            .await?;
                    info!("Subroutine spawned: {:?}", subroutine);

                    // store the instance and return it
                    let subroutine = self.repo.subroutines_create(subroutine).await?;
                    Ok(subroutine)
                }
                Err(err) => match err {
                    SubroutineDefinitionsError::NotFound(_) => {
                        Err(SubroutinesError::InvalidSubroutineDefinition(
                            input.subroutine_definition_id().into(),
                        ))
                    }
                    err => Err(SubroutinesError::from(err)),
                },
            }
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
        let msg = format!(
            "Failed to send subroutine spawn command to manager: {}",
            err
        );
        warn!("{}", msg);
        SubroutinesError::SpawnError(msg)
    })?;

    trace!("Command sent to manager.  awaiting response...");
    let res = resp_rx.await.map_err(|err| {
        let msg = format!(
            "Failed to receive response from manager to spawn request: {}",
            err
        );
        warn!("{}", err);
        SubroutinesError::SpawnError(msg)
    })?;

    trace!("Spawn response received from manager: {:?}", res);
    res.map_err(|err| {
        let msg = format!(
            "Manager returned error in response to spawn request: {}",
            err
        );
        warn!("{}", msg);
        SubroutinesError::SpawnError(msg)
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::subroutine_definitions::{
        entities::fixtures::subroutine_definition,
        fixtures::{mock_subroutine_definitions_service, MockSubroutineDefinitionsService},
        SubroutineDefinitionsError,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, Subroutine},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        worker::{fixtures::mock_subroutines_worker, MockSubroutinesWorker},
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_already_running(
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        mock_subroutines_worker: MockSubroutinesWorker,
    ) {
        subroutines_repository
            .expect_subroutines_exists()
            .return_const(Ok(true));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        let res = service
            .create(&SubroutinesCreateInput::new("test", "test", "test"))
            .await;

        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            SubroutinesError::AlreadyRunning(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_definition_does_not_exist(
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        mock_subroutines_worker: MockSubroutinesWorker,
    ) {
        subroutines_repository
            .expect_subroutines_exists()
            .return_const(Ok(false));
        mock_subroutine_definitions_service
            .expect_get()
            .return_const(Err(SubroutineDefinitionsError::NotFound("".into())));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        let res = service
            .create(&SubroutinesCreateInput::new("test", "test", "test"))
            .await;

        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            SubroutinesError::InvalidSubroutineDefinition(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_start_command_to_manager_and_adds_record(
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        mut mock_subroutines_worker: MockSubroutinesWorker,
        subroutine: Subroutine,
        subroutine_definition: SubroutineDefinition,
    ) {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);
        mock_subroutines_worker.expect_sender().return_const(cmd_tx);
        mock_subroutine_definitions_service
            .expect_get()
            .return_const(Ok(subroutine_definition));

        // projector does not exist
        subroutines_repository
            .expect_subroutines_exists()
            .return_const(Ok(false));

        // Setup fake manager
        let new_subroutine = subroutine.clone();
        tokio::spawn(async move {
            match cmd_rx.recv().await.unwrap() {
                SubroutineCommand::Spawn { resp, .. } => {
                    resp.send(Ok(new_subroutine)).unwrap();
                }
                cmd => panic!("Incorrect command received from service: {:?}", cmd),
            }
        });

        // expect creation
        subroutines_repository
            .expect_subroutines_create()
            .with(eq(subroutine.clone()))
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        service
            .create(&SubroutinesCreateInput::new("test", "test", "test"))
            .await
            .unwrap();
    }
}
