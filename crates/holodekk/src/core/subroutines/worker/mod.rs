mod spawn;
pub use spawn::SpawnError;
mod terminate;
pub use terminate::TerminationError;

use std::sync::Arc;

use log::{debug, warn};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use crate::config::HolodekkConfig;
use crate::core::projectors::entities::ProjectorEntity;
use crate::core::subroutine_definitions::entities::SubroutineDefinitionEntity;

use super::entities::SubroutineEntity;

#[derive(Debug)]
pub enum SubroutinesRequest {
    Spawn {
        projector: ProjectorEntity,
        definition: SubroutineDefinitionEntity,
        resp: tokio::sync::oneshot::Sender<std::result::Result<SubroutineEntity, SpawnError>>,
    },
    Terminate {
        subroutine: SubroutineEntity,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), TerminationError>>,
    },
}

#[derive(Debug)]
pub struct SubroutinesWorker<C>
where
    C: HolodekkConfig,
{
    config: Arc<C>,
    receiver: Receiver<SubroutinesRequest>,
}

impl<C> SubroutinesWorker<C>
where
    C: HolodekkConfig,
{
    pub fn start(config: Arc<C>, receiver: Receiver<SubroutinesRequest>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut worker = SubroutinesWorker { config, receiver };

            worker.run().await;
        })
    }

    async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(request) = self.receiver.recv() => {
                    self.process_request(request).await
                }
                else => {
                    debug!("All Subroutines worker senders closed.  Exiting ...");
                    break;
                }
            }
        }
    }

    async fn process_request(&self, request: SubroutinesRequest) {
        match request {
            SubroutinesRequest::Spawn {
                projector,
                definition,
                resp,
            } => {
                let response = self.process_spawn_request(&projector, &definition).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to Spawn request (receiver dropped)");
                }
            }
            SubroutinesRequest::Terminate { subroutine, resp } => {
                let response = self.process_terminate_request(&subroutine).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to Terminate request (receiver dropped)");
                }
            }
        }
    }

    async fn process_spawn_request(
        &self,
        projector: &ProjectorEntity,
        definition: &SubroutineDefinitionEntity,
    ) -> std::result::Result<SubroutineEntity, SpawnError> {
        self.spawn(projector, definition).await
    }

    async fn process_terminate_request(
        &self,
        subroutine: &SubroutineEntity,
    ) -> std::result::Result<(), TerminationError> {
        self.terminate(subroutine).await
    }
}

// #[cfg(test)]
// pub mod fixtures {
//     use rstest::*;

//     use super::*;

//     #[fixture]
//     pub fn mock_subroutines_worker() -> MockSubroutinesWorker {
//         MockSubroutinesWorker::default()
//     }
// }
