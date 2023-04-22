use anyhow::Context;
use async_trait::async_trait;
use log::{info, trace, warn};

use crate::core::projectors::{
    entities::ProjectorEntity,
    repositories::{ProjectorsQuery, ProjectorsRepository},
    CreateProjector, ProjectorsCreateInput, ProjectorsError, Result,
};
use crate::servers::director::{DirectorError, DirectorRequest};

use super::ProjectorsService;

#[async_trait]
impl<R> CreateProjector for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<ProjectorEntity> {
        trace!("ProjectorsService.create({:?})", input);

        // ensure a projector is not already running for this namespace
        let query = ProjectorsQuery::builder()
            .namespace_eq(input.namespace())
            .build();
        let projectors = self.repo.projectors_find(query).await?;
        if !projectors.is_empty() {
            warn!(
                "projector already running for namespace: {}",
                input.namespace
            );
            Err(ProjectorsError::AlreadyRunning(
                projectors.first().unwrap().id().to_string(),
            ))
        } else {
            // send spawn request to manager
            info!("Spawning projector for namespace: {}", input.namespace);
            let projector = self.send_spawn_request(input.namespace).await?;
            info!("Projector spawned: {:?}", projector);

            // store the projector and return it
            let projector = self.repo.projectors_create(projector).await?;
            Ok(projector)
        }
    }
}

impl<R> ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn send_spawn_request(&self, namespace: &str) -> Result<ProjectorEntity> {
        trace!("ProjectorsService::send_spawn_request({:?})", namespace);

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

        let request = DirectorRequest::SpawnProjector {
            namespace: namespace.to_string(),
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
            DirectorError::ProjectorSpawn(spawn) => ProjectorsError::from(spawn),
            DirectorError::Unexpected(unexpected) => ProjectorsError::from(unexpected),
            _ => ProjectorsError::from(anyhow::anyhow!(err.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::core::projectors::{
        entities::{fixtures::projector, ProjectorEntity},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
    };
    use crate::servers::director::DirectorRequest;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_projector_already_running(
        mut projectors_repository: MockProjectorsRepository,
        projector: ProjectorEntity,
    ) {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        projectors_repository
            .expect_projectors_find()
            .return_const(Ok(vec![projector]));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

        let res = service
            .create(&ProjectorsCreateInput {
                namespace: "existing",
            })
            .await;

        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            ProjectorsError::AlreadyRunning(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn sends_start_command_to_manager_and_adds_record(
        mut projectors_repository: MockProjectorsRepository,
        projector: ProjectorEntity,
    ) {
        let (director_tx, mut director_rx) = tokio::sync::mpsc::channel(1);

        // projector does not exist
        projectors_repository
            .expect_projectors_find()
            .return_const(Ok(vec![]));

        // Setup fake director
        let new_projector = projector.clone();
        tokio::spawn(async move {
            match director_rx.recv().await.unwrap() {
                DirectorRequest::SpawnProjector { resp, .. } => {
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

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

        service
            .create(&ProjectorsCreateInput {
                namespace: "nonexistent",
            })
            .await
            .unwrap();
    }
}
