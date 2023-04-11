mod create;
pub use create::*;

use std::sync::Arc;

use crate::config::HolodekkConfig;
use crate::repositories::ProjectorRepository;

#[derive(Clone, Debug)]
pub struct ProjectorsService<T>
where
    T: ProjectorRepository,
{
    config: Arc<HolodekkConfig>,
    repo: Arc<T>,
}

impl<T> ProjectorsService<T>
where
    T: ProjectorRepository,
{
    fn new(config: Arc<HolodekkConfig>, repo: Arc<T>) -> Self {
        Self { config, repo }
    }
}
