mod create;
pub use create::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::core::repositories::{SubroutineInstancesRepository, SubroutinesRepository};
use crate::managers::subroutine::SubroutineCommand;

#[derive(Clone, Debug)]
pub struct SubroutineInstancesService<T>
where
    T: SubroutinesRepository + SubroutineInstancesRepository,
{
    repo: Arc<T>,
    manager: Sender<SubroutineCommand>,
}

impl<T> SubroutineInstancesService<T>
where
    T: SubroutinesRepository + SubroutineInstancesRepository,
{
    pub fn new(repo: Arc<T>, manager: Sender<SubroutineCommand>) -> Self {
        Self { repo, manager }
    }
}
