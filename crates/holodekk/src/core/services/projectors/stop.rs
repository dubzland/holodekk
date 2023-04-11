use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{entities, repositories::ProjectorRepository, services::Result};
use crate::managers::projector::ProjectorCommand;

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorStopInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Stop {
    async fn stop(&self, input: ProjectorStopInput) -> Result<()>;
}

#[async_trait]
impl<T> Stop for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn stop(&self, input: ProjectorStopInput) -> Result<()> {
        let id = entities::projector::generate_id(&self.config.fleet, &input.namespace);
        let projector = self.repo.projector_get(&id).await?;

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let cmd = ProjectorCommand::Shutdown {
            projector: projector.clone(),
            resp: resp_tx,
        };
        self.manager.send(cmd).await.unwrap();
        resp_rx.await.unwrap().unwrap();
        self.repo.projector_delete(&projector.id).await?;
        Ok(())
    }
}
