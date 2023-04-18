mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod find;
pub use find::*;

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::core::services::ServiceStop;
use crate::core::subroutine_definitions::SubroutineDefinitionsServiceMethods;
use crate::core::subroutines::{repositories::SubroutinesRepository, worker::SubroutineCommand};
use crate::utils::Worker;

#[derive(Debug)]
pub struct SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
    D: SubroutineDefinitionsServiceMethods,
{
    repo: Arc<R>,
    definitions: Arc<D>,
    worker: RwLock<Option<W>>,
}

impl<R, W, D> SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
    D: SubroutineDefinitionsServiceMethods,
{
    pub fn new(repo: Arc<R>, definitions: Arc<D>, worker: W) -> Self {
        Self {
            repo,
            definitions,
            worker: RwLock::new(Some(worker)),
        }
    }

    pub fn worker(&self) -> Sender<SubroutineCommand> {
        self.worker
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .sender()
            .unwrap()
    }
}

#[async_trait]
impl<R, W, D> ServiceStop for SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn stop(&self) {
        let mut worker = self.worker.write().unwrap().take().unwrap();
        worker.stop().await;
    }
}
