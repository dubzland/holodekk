use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

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
    async fn start(&self, input: ProjectorStartInput) -> Result<Projector>;
}

#[async_trait]
impl<T> Start for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn start(&self, input: ProjectorStartInput) -> Result<Projector> {
        let id = entities::projector::generate_id(&self.fleet, &input.namespace);
        if self.repo.projector_get(&id).await.is_ok() {
            Err(Error::Duplicate)
        } else {
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            let cmd = ProjectorCommand::Spawn {
                namespace: input.namespace,
                resp: resp_tx,
            };
            self.manager.send(cmd).await.unwrap();
            let projector: Projector = resp_rx.await.unwrap()?;
            // Store the projector and return it
            let projector = self.repo.projector_create(projector).await?;
            Ok(projector)
        }
    }
}
