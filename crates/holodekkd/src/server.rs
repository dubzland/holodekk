use std::sync::Arc;

use log::debug;
use tokio::sync::mpsc::Sender;

use holodekk::{
    config::{HolodekkApiConfig, HolodekkConfig},
    core::repositories::{
        memory::{MemoryDatabase, MemoryRepository},
        RepositoryKind,
    },
    managers::projector::{ProjectorCommand, ProjectorManager},
    servers::{start_http_server, HttpServerHandle},
};

use crate::api::server::router;

pub struct HolodekkServer {
    projector_manager: ProjectorManager,
    api_server: HttpServerHandle,
}

impl HolodekkServer {
    fn new(projector_manager: ProjectorManager, api_server: HttpServerHandle) -> Self {
        Self {
            projector_manager,
            api_server,
        }
    }

    pub fn start<C>(config: Arc<C>) -> Self
    where
        C: HolodekkConfig + HolodekkApiConfig,
    {
        let repo = match config.repo_kind() {
            RepositoryKind::Memory => {
                let db = MemoryDatabase::new();
                Arc::new(MemoryRepository::new(Arc::new(db)))
            }
        };

        debug!("starting Projector Manager...");
        let projector_manager = ProjectorManager::start(config.clone());

        debug!("starting Holodekk API server...");
        let api_config = config.holodekk_api_config().clone();
        let api_server = start_http_server(
            &api_config,
            router(config, repo, projector_manager.cmd_tx()),
        );
        Self::new(projector_manager, api_server)
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.projector_manager.stop().await;
        self.api_server.stop().await.unwrap();
        Ok(())
    }

    pub fn manager_tx(&self) -> Sender<ProjectorCommand> {
        self.projector_manager.cmd_tx()
    }
}
