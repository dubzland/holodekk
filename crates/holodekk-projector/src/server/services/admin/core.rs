use tonic::{Request, Response, Status};

use crate::proto::admin::core::Core;
use crate::proto::admin::entities::{Empty, ProjectorStatus};

pub struct CoreService {}
impl CoreService {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl Core for CoreService {
    async fn status(&self, _request: Request<Empty>) -> Result<Response<ProjectorStatus>, Status> {
        Ok(Response::new(ProjectorStatus { pid: 1, port: 2 }))
    }
}
