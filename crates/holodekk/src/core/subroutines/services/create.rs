use anyhow::Context;
use async_trait::async_trait;
use log::{info, trace};

use crate::core::projectors::{entities::ProjectorEntity, ProjectorsServiceMethods};
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinitionEntity, SubroutineDefinitionsServiceMethods,
};
use crate::core::subroutines::{
    entities::SubroutineEntity,
    repositories::{SubroutinesQuery, SubroutinesRepository},
    CreateSubroutine, Result, SubroutinesCreateInput, SubroutinesError,
};
use crate::servers::director::{DirectorError, DirectorRequest};

use super::SubroutinesService;

#[async_trait]
impl<R, P, D> CreateSubroutine for SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn create<'c>(&self, input: &'c SubroutinesCreateInput<'c>) -> Result<SubroutineEntity> {
        trace!("SubroutinesService.create({:?})", input);

        // get the projector entity
        let projector = self.get_projector(input.projector_id()).await?;

        // get the subroutine definition
        let definition = self
            .get_subroutine_definition(input.subroutine_definition_id())
            .await?;

        // ensure this subroutine isn't already running in the selected namespace
        let query = SubroutinesQuery::builder()
            .for_subroutine_definition(definition.id())
            .for_projector(projector.id())
            .build();
        if self.repo.subroutines_exists(&query).await? {
            Err(SubroutinesError::AlreadyRunning)
        } else {
            // send spawn request to manager
            info!(
                "Spawning subroutine {} in namespace {}",
                definition.name(),
                projector.namespace(),
            );
            let subroutine = self
                .send_spawn_request(projector.clone(), definition.clone())
                .await?;
            info!("Subroutine spawned: {:?}", subroutine);

            // store the instance and return it
            let subroutine = self.repo.subroutines_create(subroutine).await?;
            Ok(subroutine)
        }
    }
}

impl<R, P, D> SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn send_spawn_request(
        &self,
        projector: ProjectorEntity,
        subroutine_definition: SubroutineDefinitionEntity,
    ) -> Result<SubroutineEntity> {
        trace!(
            "SubroutinesService::send_spawn_request({:?}, {:?})",
            projector,
            subroutine_definition
        );

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

        let request = DirectorRequest::SpawnSubroutine {
            projector,
            definition: subroutine_definition,
            resp: resp_tx,
        };

        trace!("request: {:?}", request);
        self.director()
            .send(request)
            .await
            .context("Failed to send Spawn request to Director")?;

        trace!("Spawn request sent to director.  Awaiting response ...");
        let response = resp_rx
            .await
            .context("Error receiving response to Spawn request from Director")?;

        trace!("Spawn response received from Director: {:?}", response);
        response.map_err(|err| match err {
            DirectorError::SubroutineSpawn(spawn) => SubroutinesError::from(spawn),
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

    use crate::core::projectors::{
        entities::{fixtures::projector, ProjectorEntity},
        fixtures::{mock_projectors_service, MockProjectorsService},
        ProjectorsError,
    };
    use crate::core::subroutine_definitions::{
        entities::{fixtures::subroutine_definition, SubroutineDefinitionEntity},
        fixtures::{mock_subroutine_definitions_service, MockSubroutineDefinitionsService},
        SubroutineDefinitionsError,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, SubroutineEntity},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_projector_does_not_exist(
        mut mock_projectors_service: MockProjectorsService,
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        subroutines_repository: MockSubroutinesRepository,
        projector: ProjectorEntity,
        subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        let get_projector_result = Err(ProjectorsError::NotFound("".into()));
        mock_projectors_service
            .expect_get()
            .return_once(move |_| get_projector_result);

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        let res = service
            .create(&SubroutinesCreateInput::new(
                projector.id(),
                subroutine_definition.id(),
            ))
            .await;

        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            SubroutinesError::InvalidProjector(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_definition_does_not_exist(
        mut mock_projectors_service: MockProjectorsService,
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        subroutines_repository: MockSubroutinesRepository,
        projector: ProjectorEntity,
        subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        let projectors_get_result = Ok(projector.clone());
        mock_projectors_service
            .expect_get()
            .return_once(move |_| projectors_get_result);

        let subroutine_definitions_get_result = Err(SubroutineDefinitionsError::NotFound(
            subroutine_definition.id().into(),
        ));
        mock_subroutine_definitions_service
            .expect_get()
            .return_once(move |_| subroutine_definitions_get_result);

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        let res = service
            .create(&SubroutinesCreateInput::new(
                projector.id(),
                subroutine_definition.id(),
            ))
            .await;

        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            SubroutinesError::InvalidSubroutineDefinition(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_subroutine_already_running(
        mut mock_projectors_service: MockProjectorsService,
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        projector: ProjectorEntity,
        subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        let projectors_get_result = Ok(projector.clone());
        mock_projectors_service
            .expect_get()
            .return_once(move |_| projectors_get_result);

        let subroutine_definitions_get_result = Ok(subroutine_definition.clone());
        mock_subroutine_definitions_service
            .expect_get()
            .return_once(move |_| subroutine_definitions_get_result);

        subroutines_repository
            .expect_subroutines_exists()
            .return_const(Ok(true));

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        let res = service
            .create(&SubroutinesCreateInput::new(
                projector.id(),
                subroutine_definition.id(),
            ))
            .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), SubroutinesError::AlreadyRunning));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_start_command_to_manager_and_adds_record(
        mut mock_projectors_service: MockProjectorsService,
        mut mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        projector: ProjectorEntity,
        subroutine_definition: SubroutineDefinitionEntity,
        subroutine: SubroutineEntity,
    ) {
        let (director_tx, mut director_rx) = tokio::sync::mpsc::channel(1);

        let projectors_get_result = Ok(projector.clone());
        mock_projectors_service
            .expect_get()
            .return_once(move |_| projectors_get_result);

        let subroutine_definitions_get_result = Ok(subroutine_definition.clone());
        mock_subroutine_definitions_service
            .expect_get()
            .return_once(move |_| subroutine_definitions_get_result);

        subroutines_repository
            .expect_subroutines_exists()
            .return_const(Ok(false));

        // Setup fake director
        let new_subroutine = subroutine.clone();
        tokio::spawn(async move {
            match director_rx.recv().await.unwrap() {
                DirectorRequest::SpawnSubroutine { resp, .. } => {
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
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        service
            .create(&SubroutinesCreateInput::new(
                projector.id(),
                subroutine_definition.id(),
            ))
            .await
            .unwrap();
    }
}
