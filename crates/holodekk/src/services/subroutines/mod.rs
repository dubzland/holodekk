mod create;
mod status;

use std::sync::Arc;

use crate::repositories::Repository;

pub struct SubroutinesService<T>
where
    T: Repository,
{
    repo: Arc<T>,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    pub fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }
}
