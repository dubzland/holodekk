use async_trait::async_trait;

use tonic::{Request, Response, Status};

use holodekk_utils::server::tonic::{TonicServerBuilder, TonicService};

use super::proto::applications::{RpcApplications, RpcApplicationsServer};
use super::proto::entities::{RpcEmpty, RpcListReply};

#[derive(Clone, Debug, Default)]
pub struct ApplicationsService {}

impl ApplicationsService {
    fn to_server() -> RpcApplicationsServer<Self> {
        RpcApplicationsServer::new(Self::default())
    }
}

#[async_trait]
impl RpcApplications for ApplicationsService {
    async fn list(
        &self,
        _request: Request<RpcEmpty>,
    ) -> std::result::Result<Response<RpcListReply>, Status> {
        let reply = RpcListReply {
            message: "Hello!".to_string(),
        };
        Ok(Response::new(reply))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProjectorApi {}

impl ProjectorApi {
    pub fn build(self) -> TonicServerBuilder<Self> {
        TonicServerBuilder::new(self)
    }
}

impl TonicService for ProjectorApi {
    fn to_router(&self) -> tonic::transport::server::Router {
        tonic::transport::Server::builder().add_service(ApplicationsService::to_server())
    }
}
