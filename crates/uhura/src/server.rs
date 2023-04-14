use std::sync::Arc;

use holodekk::{
    apis::grpc::subroutines::subroutines_api_server,
    config::{HolodekkConfig, ProjectorConfig, UhuraApiConfig},
    core::{repositories::SubroutinesRepository, services::subroutines::SubroutinesService},
};

use crate::{apis::grpc::uhura::uhura_api_server, services::UhuraService};

use holodekk::servers::{start_grpc_server, GrpcServerHandle};

pub struct UhuraServer {
    server_handle: GrpcServerHandle,
}

impl UhuraServer {
    fn new(server_handle: GrpcServerHandle) -> Self {
        Self { server_handle }
    }

    pub fn start<C, T>(config: Arc<C>, repo: Arc<T>) -> UhuraServer
    where
        C: HolodekkConfig + ProjectorConfig + UhuraApiConfig + Clone,
        T: SubroutinesRepository,
    {
        let uhura_service = Arc::new(UhuraService::new());
        let subroutines_service = Arc::new(SubroutinesService::new(repo));
        let uhura_server = tonic::transport::Server::builder()
            .add_service(uhura_api_server(uhura_service))
            .add_service(subroutines_api_server(subroutines_service));

        let server_handle = start_grpc_server(config.uhura_api_config(), uhura_server);

        Self::new(server_handle)
    }

    pub async fn stop(self) {
        self.server_handle.stop().await.unwrap();
    }
}
