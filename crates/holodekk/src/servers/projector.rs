use std::sync::Arc;

use crate::{
    apis::grpc::applications::applications_api_server, config::ProjectorApiConfig,
    core::repositories::SubroutineRepository,
};

use super::{start_grpc_server, GrpcServerHandle};

pub struct ProjectorServer<T>
where
    T: SubroutineRepository,
{
    _repository: Arc<T>,
    projector_api: GrpcServerHandle,
}

impl<T> ProjectorServer<T>
where
    T: SubroutineRepository,
{
    fn new(_repository: Arc<T>, projector_api: GrpcServerHandle) -> Self {
        Self {
            _repository,
            projector_api,
        }
    }

    pub fn start<C>(config: &C, repository: Arc<T>) -> Self
    where
        C: ProjectorApiConfig,
    {
        let projector_api = start_grpc_server(
            config.projector_api_config(),
            tonic::transport::Server::builder().add_service(applications_api_server()),
        );
        Self::new(repository, projector_api)
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.projector_api.stop().await?;
        Ok(())
    }
}
