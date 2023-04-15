use std::sync::Arc;

use log::debug;

use holodekk::{
    config::{HolodekkConfig, ProjectorConfig, UhuraApiConfig},
    core::subroutines::api::server::subroutines_api_server,
    core::subroutines::{
        self,
        repositories::{SubroutineDefinitionsRepository, SubroutinesRepository},
        services::subroutines::SubroutinesService,
        worker::SubroutinesWorker,
    },
    utils::{
        servers::{start_grpc_server, GrpcServerHandle},
        TaskHandle, Worker,
    },
};

use crate::{apis::grpc::uhura::uhura_api_server, services::UhuraService};

pub struct UhuraServerHandle {
    subroutines_worker: SubroutinesWorker,
    api_server: GrpcServerHandle,
}

impl UhuraServerHandle {
    fn new(subroutines_worker: SubroutinesWorker, api_server: GrpcServerHandle) -> Self {
        Self {
            subroutines_worker,
            api_server,
        }
    }

    pub async fn stop(mut self) -> Result<(), tonic::transport::Error> {
        self.api_server.stop().await.unwrap();
        self.subroutines_worker.stop().await;
        Ok(())
    }
}

pub fn start_uhura_server<C, R>(config: Arc<C>, repo: Arc<R>) -> UhuraServerHandle
where
    C: HolodekkConfig + ProjectorConfig + UhuraApiConfig,
    R: SubroutinesRepository + SubroutineDefinitionsRepository + 'static,
{
    debug!("starting Subroutine worker service ...");
    let subroutines_worker = subroutines::worker::start_worker(config.clone());

    debug!("starting Uhura API server...");
    let subroutines_service = Arc::new(SubroutinesService::new(
        repo,
        subroutines_worker.sender().unwrap(),
    ));
    let uhura_service = Arc::new(UhuraService::new());
    let uhura_server = tonic::transport::Server::builder()
        .add_service(uhura_api_server(uhura_service))
        .add_service(subroutines_api_server(subroutines_service));

    let api_server = start_grpc_server(config.uhura_api_config(), uhura_server);

    UhuraServerHandle::new(subroutines_worker, api_server)
}
