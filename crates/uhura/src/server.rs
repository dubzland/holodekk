use std::sync::Arc;

use log::{debug, warn};

use holodekk::utils::{
    servers::{start_grpc_server, GrpcServerHandle},
    ConnectionInfo,
};

use crate::apis::grpc::uhura::uhura_api;

pub struct Handle {
    api_server: GrpcServerHandle,
}

impl Handle {
    fn new(api_server: GrpcServerHandle) -> Self {
        Self { api_server }
    }

    pub async fn stop(self) {
        if let Err(err) = self.api_server.stop().await {
            warn!("Error stopping Uhura server: {err}");
        }
    }
}

#[must_use]
pub fn start(config: &crate::Config) -> Handle {
    debug!("starting Uhura API server...");
    let uhura_service = Arc::new(crate::Service::new());
    let uhura_server = tonic::transport::Server::builder().add_service(uhura_api(uhura_service));

    let uhura_listener = ConnectionInfo::unix(config.scene_paths().socket());

    let api_server = start_grpc_server(&uhura_listener, uhura_server);

    Handle::new(api_server)
}
