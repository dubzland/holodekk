mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod find;
pub use find::*;

mod get;
pub use get::*;

use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::config::HolodekkConfig;
use crate::core::services::{Result, ServiceStop};
use crate::utils::Worker;

use super::{repositories::ProjectorsRepository, worker::ProjectorCommand};

/// Service object for managing [Projector](super::entities::Projector) instances.
// #[derive(Debug)]
pub struct ProjectorsService<R, W>
where
    R: ProjectorsRepository,
    W: Worker<Command = ProjectorCommand>,
{
    fleet: String,
    repo: Arc<R>,
    worker: RwLock<Option<W>>,
}

impl<R, W> ProjectorsService<R, W>
where
    R: ProjectorsRepository,
    W: Worker<Command = ProjectorCommand>,
{
    pub fn new<C>(config: Arc<C>, repo: Arc<R>, worker: W) -> Self
    where
        C: HolodekkConfig,
    {
        Self {
            fleet: config.fleet().into(),
            repo,
            worker: RwLock::new(Some(worker)),
        }
    }

    pub fn worker(&self) -> tokio::sync::mpsc::Sender<ProjectorCommand> {
        self.worker
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .sender()
            .unwrap()
    }
}

#[async_trait]
impl<R, W> ServiceStop for ProjectorsService<R, W>
where
    R: ProjectorsRepository,
    W: Worker<Command = ProjectorCommand>,
{
    async fn stop(&self) -> Result<()> {
        let mut worker = self.worker.write().unwrap().take().unwrap();
        worker.stop().await;
        Ok(())
    }
}
