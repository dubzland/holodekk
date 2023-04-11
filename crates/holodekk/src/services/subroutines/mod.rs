mod create;
pub use create::*;

mod start;
pub use start::*;

mod status;
pub use status::*;

use std::sync::Arc;

use crate::config::HolodekkConfig;
use crate::repositories::Repository;

#[derive(Clone, Debug)]
pub struct SubroutinesService<T>
where
    T: Repository,
{
    config: Arc<HolodekkConfig>,
    repo: Arc<T>,
    namespace: String,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    pub fn new<S>(config: Arc<HolodekkConfig>, repo: Arc<T>, namespace: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            config,
            repo,
            namespace: namespace.into(),
        }
    }
}
