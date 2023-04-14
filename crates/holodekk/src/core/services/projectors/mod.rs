mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod exists;
pub use exists::*;

mod find;
pub use find::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::config::HolodekkConfig;
use crate::core::repositories::ProjectorsRepository;
use crate::managers::projector::ProjectorCommand;

/// Service object for managing [Projector](crate::core::entities::Projector) instances.
#[derive(Clone, Debug)]
pub struct ProjectorsService<T>
where
    T: ProjectorsRepository,
{
    fleet: String,
    repo: Arc<T>,
    manager: Sender<ProjectorCommand>,
}

impl<T> ProjectorsService<T>
where
    T: ProjectorsRepository,
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
