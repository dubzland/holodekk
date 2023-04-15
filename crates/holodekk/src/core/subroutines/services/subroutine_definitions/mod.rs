mod create;
pub use create::*;

mod start;
pub use start::*;

mod status;
pub use status::*;

use std::sync::Arc;

use crate::core::subroutines::repositories::SubroutineDefinitionsRepository;

#[derive(Clone, Debug)]
pub struct SubroutineDefinitionsService<T>
where
    T: SubroutineDefinitionsRepository,
{
    repo: Arc<T>,
}

impl<T> SubroutineDefinitionsService<T>
where
    T: SubroutineDefinitionsRepository,
{
    pub fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }
}
