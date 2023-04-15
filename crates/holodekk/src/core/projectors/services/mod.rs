mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod exists;
pub use exists::*;

mod find;
pub use find::*;

use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::*;

use crate::config::HolodekkConfig;
use crate::core::services::Result;

use super::{entities::Projector, repositories::ProjectorsRepository, worker::ProjectorCommand};

#[derive(Clone, Debug)]
pub struct ProjectorsCreateInput {
    pub namespace: String,
}

#[derive(Clone, Debug)]
pub struct ProjectorsDeleteInput {
    pub namespace: String,
}

#[derive(Clone, Debug)]
pub struct ProjectorsExistsInput {
    pub namespace: String,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ProjectorsFindInput {
    pub fleet: Option<String>,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateProjector {
    async fn create(&self, input: ProjectorsCreateInput) -> Result<Projector>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteProjector {
    async fn delete(&self, input: ProjectorsDeleteInput) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindProjectors {
    async fn find(&self, input: ProjectorsFindInput) -> Result<Vec<Projector>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProjectorExists {
    async fn exists(&self, input: ProjectorsExistsInput) -> Result<bool>;
}

/// Service object for managing [Projector](crate::core::entities::Projector) instances.
#[derive(Debug)]
pub struct ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    fleet: String,
    repo: Arc<R>,
    worker: tokio::sync::mpsc::Sender<ProjectorCommand>,
}

impl<R> ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    pub fn new<C>(
        config: Arc<C>,
        repo: Arc<R>,
        worker: tokio::sync::mpsc::Sender<ProjectorCommand>,
    ) -> Self
    where
        C: HolodekkConfig,
    {
        Self {
            fleet: config.fleet().into(),
            repo,
            worker,
        }
    }

    pub fn worker(&self) -> tokio::sync::mpsc::Sender<ProjectorCommand> {
        self.worker.clone()
    }
}
