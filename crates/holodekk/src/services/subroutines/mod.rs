mod create;
mod status;

use std::sync::Arc;

use crate::repositories::Repository;

#[derive(Clone, Debug)]
pub struct SubroutinesService<T>
where
    T: Repository,
{
    repo: Arc<T>,
    fleet: String,
    namespace: String,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    pub fn new<S>(repo: Arc<T>, fleet: S, namespace: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            repo,
            fleet: fleet.into(),
            namespace: namespace.into(),
        }
    }
}
