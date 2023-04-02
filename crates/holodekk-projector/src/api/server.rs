use tonic::{Request, Response, Status};

use super::proto::applications::{RpcApplications, RpcApplicationsServer};
use super::proto::entities::{RpcEmpty, RpcListReply};
use holodekk_utils::ApiService;

#[derive(Clone, Debug, Default)]
pub struct ApplicationsService {}

impl ApplicationsService {
    fn to_server(&self) -> RpcApplicationsServer<Self> {
        RpcApplicationsServer::new(Self::default())
    }
}

impl ApiService for ApplicationsService {
    fn to_router(&self) -> tonic::transport::server::Router {
        tonic::transport::Server::builder().add_service(self.to_server())
    }
}

#[tonic::async_trait]
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
