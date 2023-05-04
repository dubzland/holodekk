use std::sync::Arc;

use log::debug;

use holodekk::utils::{server, ConnectionInfo, Server};

use crate::apis::grpc::uhura::uhura_api;

#[must_use]
pub fn start(config: &crate::Config) -> server::grpc::Handle {
    debug!("starting Uhura API server...");
    let uhura_service = Arc::new(crate::Service::new());
    let uhura_server = tonic::transport::Server::builder().add_service(uhura_api(uhura_service));

    let uhura_listener = ConnectionInfo::unix(config.scene_paths().socket());

    server::Grpc::start(&uhura_listener, uhura_server)
}
