use tonic::{Request, Response, Status};

use super::proto::{entities::RpcSubroutineList, RpcSubroutines, RpcSubroutinesServer};
use crate::proto::common::RpcEmpty;

#[derive(Clone, Debug, Default)]
pub struct SubroutinesApi {}

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

pub fn subroutines_api() -> RpcSubroutinesServer<SubroutinesApi> {
    RpcSubroutinesServer::new(SubroutinesApi::default())
}
