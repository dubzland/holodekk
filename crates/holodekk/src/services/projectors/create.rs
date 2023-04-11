use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::entities::Projector;
use crate::repositories::ProjectorRepository;
use crate::services::{Error, Result};

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorCreateInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create {
    async fn create(&self, input: ProjectorCreateInput) -> Result<Projector>;
}

#[async_trait]
impl<T> Create for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn create(&self, input: ProjectorCreateInput) -> Result<Projector> {
        let id = crate::entities::projector::generate_id(&self.config.fleet, &input.namespace);
        if self.repo.projector_get(&id).await.is_ok() {
            return Err(Error::Duplicate);
        } else {
            todo!()
        }
    }
}
