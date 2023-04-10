mod create;
pub use create::SubroutineCreateInput;

mod status;

use std::path::PathBuf;
use std::sync::Arc;

use crate::repositories::Repository;
use crate::HolodekkConfig;

#[derive(Clone, Debug)]
pub struct SubroutinesService<T>
where
    T: Repository,
{
    config: Arc<HolodekkConfig>,
    repo: Arc<T>,
    root: PathBuf,
    namespace: String,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    pub fn new<S, P>(config: Arc<HolodekkConfig>, repo: Arc<T>, namespace: S, root: P) -> Self
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            config,
            repo,
            namespace: namespace.into(),
            root: root.into(),
        }
    }
}
