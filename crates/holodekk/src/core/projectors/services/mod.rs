mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod find;
pub use find::*;

mod get;
pub use get::*;

use std::sync::Arc;

use crate::config::HolodekkConfig;

use super::{repositories::ProjectorsRepository, worker::ProjectorCommand};

/// Service object for managing [Projector](super::entities::Projector) instances.
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
