mod all;
pub use all::*;

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
    fleet: String,
    repo: Arc<T>,
    manager: Sender<ProjectorCommand>,
}

impl<T> ProjectorsService<T>
where
    T: ProjectorRepository,
{
    pub fn new<C>(config: Arc<C>, repo: Arc<T>, manager: Sender<ProjectorCommand>) -> Self
    where
        C: HolodekkConfig,
    {
        Self {
            fleet: config.fleet().to_string(),
            repo,
            manager,
        }
    }
}
