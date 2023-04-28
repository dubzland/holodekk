use std::sync::Arc;

use log::debug;

use holodekk_common::{
    config::{HolodekkPaths, ProjectorConfig, UhuraApiConfig},
    utils::{
        servers::{start_grpc_server, GrpcServerHandle},
        ConnectionInfo,
    },
};

use crate::{apis::grpc::uhura::uhura_api_server, services::UhuraService};

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

pub fn start_uhura_server<C>(config: Arc<C>) -> UhuraServerHandle
where
    C: HolodekkPaths + ProjectorConfig + UhuraApiConfig,
{
    debug!("starting Uhura API server...");
    let uhura_service = Arc::new(UhuraService::new());
    let uhura_server =
        tonic::transport::Server::builder().add_service(uhura_api_server(uhura_service));

    let mut socket_path = config.projector_path().to_owned();
    socket_path.push("uhura.sock");
    let uhura_listener = ConnectionInfo::unix(socket_path);

    let api_server = start_grpc_server(&uhura_listener, uhura_server);

    UhuraServerHandle::new(api_server)
}
