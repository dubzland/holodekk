use std::sync::Arc;

use holodekk::{
    apis::grpc::subroutines::subroutines_api_server, config::HolodekkConfig,
    repositories::SubroutineRepository, services::subroutines::SubroutinesService,
    utils::ConnectionInfo,
};

use crate::{apis::grpc::uhura::uhura_api_server, services::UhuraService};

use holodekk::servers::{start_grpc_server, GrpcServerHandle};

pub struct UhuraServer<T>
where
    T: SubroutineRepository,
{
    config: Arc<HolodekkConfig>,
    namespace: String,
    repository: Arc<T>,
    server_handle: Option<GrpcServerHandle>,
}

impl<T> UhuraServer<T>
where
    T: SubroutineRepository,
{
    pub fn new<S>(config: Arc<HolodekkConfig>, namespace: S, repository: Arc<T>) -> Self
    where
        S: AsRef<str> + Into<String>,
    {
        Self {
            config,
            namespace: namespace.into(),
            repository,
            server_handle: None,
        }
    }

    pub fn start(&mut self, listener_config: ConnectionInfo) {
        let uhura_service = Arc::new(UhuraService::new());
        let subroutines_service = Arc::new(SubroutinesService::new(
            self.config.clone(),
            self.repository.clone(),
            &self.namespace,
        ));
        let uhura_server = tonic::transport::Server::builder()
            .add_service(uhura_api_server(uhura_service))
            .add_service(subroutines_api_server(subroutines_service));

        let server_handle = start_grpc_server(&listener_config, uhura_server);

        self.server_handle = Some(server_handle);
    }

    pub async fn stop(self) {
        self.server_handle.unwrap().stop().await.unwrap();
    }
}
