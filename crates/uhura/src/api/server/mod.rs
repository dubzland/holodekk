use tonic::{Request, Response, Status};

use holodekk_utils::ApiService;

use super::proto::core::{Core, CoreServer};
use super::proto::entities::{Empty, ProjectorStatus, SubroutineList};
use super::proto::subroutines::{Subroutines, SubroutinesServer};

#[derive(Clone, Debug, Default)]
pub struct CoreService {}

impl CoreService {
    fn to_server() -> CoreServer<Self> {
        CoreServer::new(Self::default())
    }
}

#[tonic::async_trait]
impl Core for CoreService {
    async fn status(
        &self,
        _request: Request<Empty>,
    ) -> std::result::Result<Response<ProjectorStatus>, Status> {
        let reply = ProjectorStatus { pid: 1, port: 1234 };
        Ok(Response::new(reply))
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubroutinesService {}

impl SubroutinesService {
    fn to_server() -> SubroutinesServer<Self> {
        SubroutinesServer::new(Self::default())
    }
}

#[tonic::async_trait]
impl Subroutines for SubroutinesService {
    async fn list(
        &self,
        _request: Request<Empty>,
    ) -> std::result::Result<Response<SubroutineList>, Status> {
        let reply = SubroutineList {
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
