use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{entities, repositories::ProjectorRepository};

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorExistsInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Exists {
    async fn exists(&self, input: ProjectorExistsInput) -> bool;
}

#[async_trait]
impl<T> Exists for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn exists(&self, input: ProjectorExistsInput) -> bool {
        let id = entities::projector::generate_id(&self.fleet, &input.namespace);
        self.repo.projector_exists(&id).await
    }
}
