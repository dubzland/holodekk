use async_trait::async_trait;
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::Sender;

use crate::core::repositories::{RepositoryError, RepositoryId};
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, SubroutineDefinitionsGetInput,
    SubroutineDefinitionsServiceMethods,
};
use crate::core::subroutines::{
    entities::Subroutine, repositories::SubroutinesRepository, DeleteSubroutine, Result,
    SubroutinesDeleteInput, SubroutinesError,
};
use crate::utils::Worker;

use super::{SubroutineCommand, SubroutinesService};

#[async_trait]
impl<R, W, D> DeleteSubroutine for SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
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

        // get the definition
        let definition = self
            .definitions
            .get(&SubroutineDefinitionsGetInput::new(
                subroutine.subroutine_definition_id(),
            ))
            .await?;

        // send the shutdown command to the manager
        info!(
            "Shutting down subroutine: {}:({})",
            input.id(),
            subroutine.namespace(),
        );
        send_shutdown_command(self.worker(), subroutine.clone(), definition.clone()).await?;
        info!(
            "Subroutine shutdown complete: {}:({})",
            input.id(),
            subroutine.namespace(),
        );

        // remove subroutine from the repository
        self.repo.subroutines_delete(&subroutine.id()).await?;
        Ok(())
    }
}

async fn send_shutdown_command(
    sender: Sender<SubroutineCommand>,
    subroutine: Subroutine,
    definition: SubroutineDefinition,
) -> Result<()> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let cmd = SubroutineCommand::Shutdown {
        subroutine: subroutine.clone(),
        definition,
        resp: resp_tx,
    };
    debug!("command: {:?}", cmd);
    sender.send(cmd).await.map_err(|err| {
        let msg = format!(
            "Failed to send subroutine shutdown command to manager: {}",
            err
        );
        warn!("{}", msg);
        SubroutinesError::Shutdown(msg)
    })?;

    trace!("Command sent to manager.  awaiting response...");
    let res = resp_rx.await.map_err(|err| {
        let msg = format!(
            "Failed to receive response from manager to shutdown request: {}",
            err
        );
        warn!("{}", msg);
        SubroutinesError::Shutdown(msg)
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
            Err(SubroutinesError::Shutdown(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::repositories::RepositoryError;
    use crate::core::subroutine_definitions::{
        entities::{fixtures::subroutine_definition, SubroutineDefinition},
        fixtures::{mock_subroutine_definitions_service, MockSubroutineDefinitionsService},
        SubroutineDefinitionsError,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, Subroutine},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        worker::{fixtures::mock_subroutines_worker, MockSubroutinesWorker},
        SubroutinesError,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_subroutine(
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mock_subroutines_worker: MockSubroutinesWorker,
        mut subroutines_repository: MockSubroutinesRepository,
    ) {
        subroutines_repository
            .expect_subroutines_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );
        let res = service
            .delete(&SubroutinesDeleteInput::new("nonexistent"))
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), SubroutinesError::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_subroutine_definition(
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mock_subroutines_worker: MockSubroutinesWorker,
        mut subroutines_repository: MockSubroutinesRepository,
    ) {
        subroutines_repository
            .expect_subroutines_get()
            .return_const(Err(RepositoryError::NotFound("".into())));

        mock_subroutine_definitions_service
            .expect_get()
            .return_const(Err(SubroutineDefinitionsError::NotFound("".to_string())));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );
        let res = service
            .delete(&SubroutinesDeleteInput::new("nonexistent"))
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), SubroutinesError::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_stop_command_to_manager_and_removes_record(
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut mock_subroutines_worker: MockSubroutinesWorker,
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine_definition: SubroutineDefinition,
        subroutine: Subroutine,
    ) {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);
        mock_subroutines_worker.expect_sender().return_const(cmd_tx);

        // subroutine exists
        subroutines_repository
            .expect_subroutines_get()
            .return_const(Ok(subroutine.clone()));

        mock_subroutine_definitions_service
            .expect_get()
            .return_const(Ok(subroutine_definition));

        // Setup fake manager
        tokio::spawn(async move {
            match cmd_rx.recv().await.unwrap() {
                SubroutineCommand::Shutdown { resp, .. } => {
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
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        service
            .delete(&SubroutinesDeleteInput::new(&subroutine.id()))
            .await
            .unwrap();
    }
}
