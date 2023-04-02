use std::sync::Arc;

use tonic::{Request, Response, Status};

use holodekk_utils::ApiService;

use crate::services::CoreService;

use super::proto::core::{RpcCore, RpcCoreServer};
use super::proto::entities::{RpcEmpty, RpcProjectorStatus, RpcSubroutineList};
use super::proto::subroutines::{RpcSubroutines, RpcSubroutinesServer};

#[derive(Clone, Debug)]
pub struct CoreApi {
    core_service: Arc<CoreService>,
}

impl CoreApi {
    fn new(core_service: Arc<CoreService>) -> Self {
        Self { core_service }
    }

    fn to_server(core_service: Arc<CoreService>) -> RpcCoreServer<Self> {
        RpcCoreServer::new(Self::new(core_service))
    }
}

#[tonic::async_trait]
impl RpcCore for CoreApi {
    async fn status(
        &self,
        _request: Request<RpcEmpty>,
    ) -> std::result::Result<Response<RpcProjectorStatus>, Status> {
        let status = self.core_service.status()?;
        let reply: RpcProjectorStatus = status.into();
        Ok(Response::new(reply))
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubroutinesApi {}

impl SubroutinesApi {
    fn to_server() -> RpcSubroutinesServer<Self> {
        RpcSubroutinesServer::new(Self::default())
    }
}

#[tonic::async_trait]
impl RpcSubroutines for SubroutinesApi {
    async fn list(
        &self,
        _request: Request<RpcEmpty>,
    ) -> std::result::Result<Response<RpcSubroutineList>, Status> {
        let reply = RpcSubroutineList {
            subroutines: vec![],
        };
        Ok(Response::new(reply))
    }
}

#[derive(Clone, Debug, Default)]
pub struct UhuraApi {
    core_service: Arc<CoreService>,
}

impl UhuraApi {
    pub fn new(core_service: CoreService) -> Self {
        Self {
            core_service: Arc::new(core_service),
        }
    }
}

impl ApiService for UhuraApi {
    fn to_router(&self) -> tonic::transport::server::Router {
        tonic::transport::Server::builder()
            .add_service(CoreApi::to_server(self.core_service.clone()))
            .add_service(SubroutinesApi::to_server())
    }
}
