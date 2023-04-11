mod start;
pub use start::*;

mod stop;
pub use stop::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::config::HolodekkConfig;
use crate::core::repositories::ProjectorRepository;
use crate::managers::projector::ProjectorCommand;

#[derive(Clone, Debug)]
pub struct ProjectorsService<T>
where
    T: ProjectorRepository,
{
    config: Arc<HolodekkConfig>,
    repo: Arc<T>,
    manager: Sender<ProjectorCommand>,
}

impl<T> ProjectorsService<T>
where
    T: ProjectorRepository,
{
    pub fn new(
        config: Arc<HolodekkConfig>,
        repo: Arc<T>,
        manager: Sender<ProjectorCommand>,
    ) -> Self {
        Self {
            config,
            repo,
            manager,
        }
    }
}
