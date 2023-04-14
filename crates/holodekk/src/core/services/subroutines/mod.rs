mod create;
pub use create::*;

mod start;
pub use start::*;

mod status;
pub use status::*;

use std::sync::Arc;

use crate::core::repositories::SubroutinesRepository;

#[derive(Clone, Debug)]
pub struct SubroutinesService<T>
where
    T: SubroutinesRepository,
{
    repo: Arc<T>,
}

impl<T> SubroutinesService<T>
where
    T: SubroutinesRepository,
{
    pub fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }
}
