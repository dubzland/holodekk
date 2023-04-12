mod create;
pub use create::*;

mod start;
pub use start::*;

mod status;
pub use status::*;

use std::sync::Arc;

use crate::config::{HolodekkConfig, ProjectorConfig};
use crate::core::repositories::SubroutineRepository;

#[derive(Clone, Debug)]
pub struct SubroutinesService<T>
where
    T: SubroutineRepository,
{
    fleet: String,
    repo: Arc<T>,
    namespace: String,
}

impl<T> SubroutinesService<T>
where
    T: SubroutineRepository,
{
    pub fn new<C>(config: &C, repo: Arc<T>) -> Self
    where
        C: HolodekkConfig + ProjectorConfig,
    {
        Self {
            fleet: config.fleet().into(),
            repo,
            namespace: config.namespace().into(),
        }
    }
}
