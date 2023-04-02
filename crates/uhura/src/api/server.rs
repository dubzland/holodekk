use tonic::{Request, Response, Status};

use holodekk_utils::ApiService;

use super::proto::core::{RpcCore, RpcCoreServer};
use super::proto::entities::{RpcEmpty, RpcProjectorStatus, RpcSubroutineList};
use super::proto::subroutines::{RpcSubroutines, RpcSubroutinesServer};

#[derive(Clone, Debug, Default)]
pub struct CoreService {}

impl CoreService {
    fn to_server() -> RpcCoreServer<Self> {
        RpcCoreServer::new(Self::default())
    }
}

#[tonic::async_trait]
impl RpcCore for CoreService {
    async fn status(
        &self,
        _request: Request<RpcEmpty>,
    ) -> std::result::Result<Response<RpcProjectorStatus>, Status> {
        let reply = RpcProjectorStatus { pid: 1, port: 1234 };
        Ok(Response::new(reply))
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubroutinesService {}

impl SubroutinesService {
    fn to_server() -> RpcSubroutinesServer<Self> {
        RpcSubroutinesServer::new(Self::default())
    }
}

#[tonic::async_trait]
impl RpcSubroutines for SubroutinesService {
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
pub struct UhuraApi {}

impl UhuraApi {}

impl ApiService for UhuraApi {
    fn to_router(&self) -> tonic::transport::server::Router {
        tonic::transport::Server::builder()
            .add_service(CoreService::to_server())
            .add_service(SubroutinesService::to_server())
    }
}
