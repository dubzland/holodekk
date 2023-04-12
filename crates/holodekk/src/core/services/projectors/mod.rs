mod all;
pub use all::*;

mod exists;
pub use exists::*;

mod start;
pub use start::*;

mod stop;
pub use stop::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::config::HolodekkConfig;
use crate::core::repositories::ProjectorRepository;
use crate::managers::projector::ProjectorCommand;

/// Service object for managing [Projector](crate::core::entities::Projector) instances.
///
/// This service acts as an interface to the repository as well as the actual projector
/// manager.  Any interaction with the above should always be done via this service to ensure data
/// integrity and to prevent synchronization issues.
///
/// # Examples
///
/// ```rust,no_run
/// # use std::path::PathBuf;
/// # use std::sync::Arc;
/// # use holodekk::config::HolodekkConfig;
/// # use holodekk::core::repositories::{memory::{MemoryDatabase, MemoryRepository}, RepositoryKind};
/// # use holodekk::managers::projector::ProjectorCommand;
/// use holodekk::core::services::projectors::{
///     ProjectorsService,
///     ProjectorStartInput,
///     Start
/// };
/// # #[derive(Clone)]
/// # struct Config { root_path: PathBuf, bin_path: PathBuf }
/// # impl HolodekkConfig for Config {
/// # fn fleet(&self) -> &str { "test" }
/// # fn root_path(&self) -> &PathBuf { &self.root_path }
/// # fn bin_path(&self) -> &PathBuf { &self.bin_path }
/// # fn repo_kind(&self) -> RepositoryKind { RepositoryKind::Memory }
/// # }
/// # #[tokio::main]
/// # async fn main() {
/// # let (manager, cmd_rx) = tokio::sync::mpsc::channel(1);
/// # let repo = Arc::new(MemoryRepository::new(Arc::new(MemoryDatabase::default())));
/// # let config = Arc::new(
/// #     Config{ root_path: PathBuf::from("/tmp"), bin_path: PathBuf::from("/tmp")}
/// # );
/// // let config = ...;
/// // let repo = ...;
/// // let manager = repository_manager.cmd_tx();
/// let service = ProjectorsService::new(config, repo, manager);
/// service.start(ProjectorStartInput{ namespace: "test".to_string() }).await.unwrap();
/// # }
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
