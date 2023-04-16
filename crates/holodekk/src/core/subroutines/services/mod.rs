mod create;
pub use create::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::core::subroutine_definitions::repositories::SubroutineDefinitionsRepository;
use crate::core::subroutines::{repositories::SubroutinesRepository, worker::SubroutineCommand};

#[derive(Debug)]
pub struct SubroutinesService<R>
where
    R: SubroutinesRepository + SubroutineDefinitionsRepository,
{
    repo: Arc<R>,
    worker: tokio::sync::mpsc::Sender<SubroutineCommand>,
}

impl<R> SubroutinesService<R>
where
    R: SubroutinesRepository + SubroutineDefinitionsRepository,
{
    pub fn new(repo: Arc<R>, worker: Sender<SubroutineCommand>) -> Self {
        Self { repo, worker }
    }

    pub fn worker(&self) -> Sender<SubroutineCommand> {
        self.worker.clone()
    }
}
