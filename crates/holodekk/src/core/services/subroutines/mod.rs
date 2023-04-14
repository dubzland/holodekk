mod create;
pub use create::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::core::repositories::{SubroutineDefinitionsRepository, SubroutinesRepository};
use crate::managers::subroutine::SubroutineCommand;

#[derive(Clone, Debug)]
pub struct SubroutinesService<T>
where
    T: SubroutinesRepository + SubroutineDefinitionsRepository,
{
    repo: Arc<T>,
    manager: Sender<SubroutineCommand>,
}

impl<T> SubroutinesService<T>
where
    T: SubroutinesRepository + SubroutineDefinitionsRepository,
{
    pub fn new(repo: Arc<T>, manager: Sender<SubroutineCommand>) -> Self {
        Self { repo, manager }
    }
}
