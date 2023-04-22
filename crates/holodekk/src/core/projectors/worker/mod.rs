mod spawn;
pub use spawn::SpawnError;
mod terminate;
pub use terminate::TerminationError;

use std::sync::Arc;

use log::{debug, warn};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use crate::config::HolodekkConfig;

use super::entities::ProjectorEntity;

#[derive(Debug)]
pub enum ProjectorsRequest {
    /// Spawn a new projector
    Spawn {
        namespace: String,
        resp: tokio::sync::oneshot::Sender<std::result::Result<ProjectorEntity, SpawnError>>,
    },
    /// Terminate a running projector
    Terminate {
        projector: ProjectorEntity,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), TerminationError>>,
    },
}

pub struct ProjectorsWorker<C>
where
    C: HolodekkConfig,
{
    config: Arc<C>,
    receiver: Receiver<ProjectorsRequest>,
}

impl<C> ProjectorsWorker<C>
where
    C: HolodekkConfig,
{
    pub fn start(config: Arc<C>, receiver: Receiver<ProjectorsRequest>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut worker = ProjectorsWorker { config, receiver };

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
                    debug!("All Projectors worker senders closed.  Exiting ...");
                    break;
                }
            }
        }
    }

    async fn process_request(&self, request: ProjectorsRequest) {
        match request {
            ProjectorsRequest::Spawn { namespace, resp } => {
                let response = self.process_spawn_request(&namespace).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to Spawn request (receiver dropped)");
                }
            }
            ProjectorsRequest::Terminate { projector, resp } => {
                let response = self.process_terminate_request(&projector).await;
                if resp.send(response).is_err() {
                    warn!("Failed to respond to Terminate request (receiver dropped)");
                }
            }
        }
    }

    async fn process_spawn_request(
        &self,
        namespace: &str,
    ) -> std::result::Result<ProjectorEntity, SpawnError> {
        self.spawn(namespace).await
    }

    async fn process_terminate_request(
        &self,
        projector: &ProjectorEntity,
    ) -> std::result::Result<(), TerminationError> {
        self.terminate(projector).await
    }
}
