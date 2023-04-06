use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::services::CoreService;

use crate::proto::entities::{RpcEmpty, RpcProjectorStatus};
use crate::proto::{RpcCore, RpcCoreServer};

#[derive(Clone, Debug)]
pub struct CoreApi {
    core_service: Arc<CoreService>,
}

#[tonic::async_trait]
impl RpcCore for CoreApi {
    async fn status(
        &self,
        _request: Request<RpcEmpty>,
    ) -> std::result::Result<Response<RpcProjectorStatus>, Status> {
        let status = self.core_service.status().unwrap();
        let reply: RpcProjectorStatus = status.into();
        Ok(Response::new(reply))
    }
}

pub fn core_api(core_service: Arc<CoreService>) -> RpcCoreServer<CoreApi> {
    RpcCoreServer::new(CoreApi { core_service })
}
