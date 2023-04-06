use tonic::{Request, Response, Status};

use super::proto::entities::{RpcEmpty, RpcListReply};
use super::proto::{RpcApplications, RpcApplicationsServer};

#[derive(Clone, Debug, Default)]
pub struct ApplicationsApi {}

#[tonic::async_trait]
impl RpcApplications for ApplicationsApi {
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

pub fn applications_api() -> RpcApplicationsServer<ApplicationsApi> {
    RpcApplicationsServer::new(ApplicationsApi::default())
}
