use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::repositories::Repository;
use crate::services::SubroutinesService;

use super::proto::{
    entities::{RpcStatusRequest, RpcSubroutineStatus},
    RpcSubroutines, RpcSubroutinesServer,
};

#[derive(Clone, Debug)]
pub struct SubroutinesApiServer<T>
where
    T: Repository,
{
    service: Arc<SubroutinesService<T>>,
}

impl<T> SubroutinesApiServer<T>
where
    T: Repository,
{
    pub fn new(service: Arc<SubroutinesService<T>>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl<T> RpcSubroutines for SubroutinesApiServer<T>
where
    T: Repository,
{
    async fn status(
        &self,
        request: Request<RpcStatusRequest>,
    ) -> std::result::Result<Response<RpcSubroutineStatus>, Status> {
        let status = self.service.status(&request.into_inner().name).await?;
        let response: RpcSubroutineStatus = status.into();

        Ok(Response::new(response))
    }
}

pub fn subroutines_api_server<T>(
    service: Arc<SubroutinesService<T>>,
) -> RpcSubroutinesServer<SubroutinesApiServer<T>>
where
    T: Repository,
{
    RpcSubroutinesServer::new(SubroutinesApiServer::new(service))
}
