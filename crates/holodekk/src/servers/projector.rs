use std::sync::Arc;

use log::debug;

use crate::{apis::grpc::applications::applications_api_server, config::ProjectorApiConfig};

use super::{start_grpc_server, GrpcServerHandle};

pub struct ProjectorServer {
    api_server: GrpcServerHandle,
}

impl ProjectorServer {
    fn new(api_server: GrpcServerHandle) -> Self {
        Self { api_server }
    }

    pub fn start<C>(config: Arc<C>) -> Self
    where
        C: ProjectorApiConfig,
    {
        debug!("starting Projector API server...");
        let api_config = config.projector_api_config().clone();
        let api_server = start_grpc_server(
            &api_config,
            tonic::transport::Server::builder().add_service(applications_api_server()),
        );
        Self::new(api_server)
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.api_server.stop().await?;
        Ok(())
    }
}
