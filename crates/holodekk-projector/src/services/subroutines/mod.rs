mod create;

use std::sync::Arc;

use crate::repository::Repository;

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
