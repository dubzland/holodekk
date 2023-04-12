use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{entities::Projector, repositories::ProjectorRepository, services::Result};

use super::ProjectorsService;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait All {
    async fn all(&self) -> Result<Vec<Projector>>;
}

#[async_trait]
impl<T> All for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn all(&self) -> Result<Vec<Projector>> {
        let projectors = self.repo.projector_all().await?;
        Ok(projectors)
    }
}
