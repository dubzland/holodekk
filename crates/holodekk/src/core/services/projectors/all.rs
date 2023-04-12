use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{entities::Projector, repositories::ProjectorRepository, services::Result};

use super::ProjectorsService;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait All {
    /// Returns the list of all currently active [Projector] instances
    async fn all(&self) -> Result<Vec<Projector>>;
}

#[async_trait]
impl<T> All for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn all(&self) -> Result<Vec<Projector>> {
        trace!("ProjectorsService.all()");
        let projectors = self.repo.projector_get_all().await?;
        Ok(projectors)
    }
}
