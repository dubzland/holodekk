use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    apis::grpc::applications::applications_api_server, repositories::SubroutineRepository,
    utils::ConnectionInfo,
};

use super::{start_grpc_server, GrpcServerHandle};

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectorConfig {
    pub fleet: String,
    pub namespace: String,
    pub root: PathBuf,
    pub api_config: ConnectionInfo,
}

pub struct ProjectorServer<T>
where
    T: SubroutineRepository,
{
    _repository: Arc<T>,
    projector_api: GrpcServerHandle,
    // projector_tx: tokio::sync::mpsc::Sender<ProjectorCommand>,
    // projector_handler: Option<ServerHandle<()>>,
}

impl<T> ProjectorServer<T>
where
    T: SubroutineRepository,
{
    pub fn new(_repository: Arc<T>, projector_api: GrpcServerHandle) -> Self {
        Self {
            _repository,
            projector_api,
        }
    }

    pub fn start(config: &ProjectorConfig, repository: Arc<T>) -> Self {
        let projector_api = start_grpc_server(
            &config.api_config,
            tonic::transport::Server::builder().add_service(applications_api_server()),
        );
        Self::new(repository, projector_api)
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.projector_api.stop().await?;
        Ok(())
    }
}
