use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::proto::entities::{RpcUhuraStatus, RpcUhuraStatusRequest};
use super::proto::{RpcUhura, RpcUhuraServer};

#[derive(Clone, Debug)]
pub struct Server {
    service: Arc<crate::Service>,
}

#[tonic::async_trait]
impl RpcUhura for Server {
    async fn status(
        &self,
        _request: Request<RpcUhuraStatusRequest>,
    ) -> std::result::Result<Response<RpcUhuraStatus>, Status> {
        let status = self.service.status();
        let reply: RpcUhuraStatus = status.into();
        Ok(Response::new(reply))
    }
}

pub fn uhura_api(service: Arc<crate::Service>) -> RpcUhuraServer<Server> {
    RpcUhuraServer::new(Server { service })
}
