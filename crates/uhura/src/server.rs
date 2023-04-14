use std::sync::Arc;

use log::debug;
use tokio::sync::mpsc::Sender;

use holodekk::{
    apis::grpc::subroutines::subroutines_api_server,
    config::{HolodekkConfig, ProjectorConfig, UhuraApiConfig},
    core::{
        repositories::{
            memory::{MemoryDatabase, MemoryRepository},
            RepositoryKind,
        },
        services::subroutines::SubroutinesService,
    },
    managers::subroutine::{SubroutineCommand, SubroutineManager},
    servers::{start_grpc_server, GrpcServerHandle},
};

use crate::{apis::grpc::uhura::uhura_api_server, services::UhuraService};

pub struct UhuraServer {
    subroutine_manager: SubroutineManager,
    api_server: GrpcServerHandle,
}

impl UhuraServer {
    fn new(subroutine_manager: SubroutineManager, api_server: GrpcServerHandle) -> Self {
        Self {
            subroutine_manager,
            api_server,
        }
    }

    pub fn start<C>(config: Arc<C>) -> UhuraServer
    where
        C: HolodekkConfig + ProjectorConfig + UhuraApiConfig + Clone,
    {
        let repo = match config.repo_kind() {
            RepositoryKind::Memory => {
                let db = MemoryDatabase::new();
                Arc::new(MemoryRepository::new(Arc::new(db)))
            }
        };

        debug!("starting Subroutine Manager...");
        let subroutine_manager = SubroutineManager::start(config.clone());

        debug!("starting Uhura API server...");
        let uhura_service = Arc::new(UhuraService::new());
        let subroutines_service =
            Arc::new(SubroutinesService::new(repo, subroutine_manager.cmd_tx()));
        let uhura_server = tonic::transport::Server::builder()
            .add_service(uhura_api_server(uhura_service))
            .add_service(subroutines_api_server(subroutines_service));

        let api_server = start_grpc_server(config.uhura_api_config(), uhura_server);

        Self::new(subroutine_manager, api_server)
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.subroutine_manager.stop().await;
        self.api_server.stop().await.unwrap();
        Ok(())
    }

    pub fn manager_tx(&self) -> Sender<SubroutineCommand> {
        self.subroutine_manager.cmd_tx()
    }
}
