use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::services::UhuraService;

use super::proto::entities::{RpcEmpty, RpcUhuraStatus};
use super::proto::{RpcUhura, RpcUhuraServer};

#[derive(Clone, Debug)]
pub struct UhuraApiServer {
    uhura_service: Arc<UhuraService>,
}

#[tonic::async_trait]
impl RpcUhura for UhuraApiServer {
    async fn status(
        &self,
        _request: Request<RpcEmpty>,
    ) -> std::result::Result<Response<RpcUhuraStatus>, Status> {
        let status = self.uhura_service.status().unwrap();
        let reply: RpcUhuraStatus = status.into();
        Ok(Response::new(reply))
    }
}

pub fn uhura_api_server(uhura_service: Arc<UhuraService>) -> RpcUhuraServer<UhuraApiServer> {
    RpcUhuraServer::new(UhuraApiServer { uhura_service })
}
