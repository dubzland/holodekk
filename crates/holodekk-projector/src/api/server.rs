use tonic::{Request, Response, Status};

use super::proto::applications::{Applications, ApplicationsServer};
use super::proto::entities::{Empty, ListReply};
use holodekk_utils::ApiService;

#[derive(Clone, Debug, Default)]
pub struct ApplicationsService {}

impl ApplicationsService {
    fn to_server(&self) -> ApplicationsServer<Self> {
        ApplicationsServer::new(Self::default())
    }
}

impl ApiService for ApplicationsService {
    fn to_router(&self) -> tonic::transport::server::Router {
        tonic::transport::Server::builder().add_service(self.to_server())
    }
}

#[tonic::async_trait]
impl Applications for ApplicationsService {
    async fn list(
        &self,
        _request: Request<Empty>,
    ) -> std::result::Result<Response<ListReply>, Status> {
        let reply = ListReply {
            message: "Hello!".to_string(),
        };
        Ok(Response::new(reply))
    }
}
