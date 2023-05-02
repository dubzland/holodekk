use std::sync::Arc;

use log::debug;

use holodekk::utils::{
    servers::{start_grpc_server, GrpcServerHandle},
    ConnectionInfo,
};

use crate::{apis::grpc::uhura::uhura_api_server, config::UhuraConfig, services::UhuraService};

pub struct UhuraServerHandle {
    api_server: GrpcServerHandle,
}

impl UhuraServerHandle {
    fn new(api_server: GrpcServerHandle) -> Self {
        Self { api_server }
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.api_server.stop().await.unwrap();
        Ok(())
    }
}

pub fn start_uhura_server(config: Arc<UhuraConfig>) -> UhuraServerHandle {
    debug!("starting Uhura API server...");
    let uhura_service = Arc::new(UhuraService::new());
    let uhura_server =
        tonic::transport::Server::builder().add_service(uhura_api_server(uhura_service));

    let uhura_listener = ConnectionInfo::unix(config.scene_paths().socket());

    let api_server = start_grpc_server(&uhura_listener, uhura_server);

    UhuraServerHandle::new(api_server)
}
