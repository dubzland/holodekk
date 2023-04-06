use tonic::transport::Channel;

use crate::entities::ProjectorStatus;
use holodekk::errors::grpc::GrpcClientResult;

use crate::api::proto::entities::RpcEmpty;
use crate::api::proto::RpcCoreClient;

#[derive(Clone, Debug)]
pub struct CoreClient {
    inner: RpcCoreClient<Channel>,
}

impl CoreClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcCoreClient::new(channel),
        }
    }

    pub async fn status(&self) -> GrpcClientResult<ProjectorStatus> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcEmpty {});
        let response = client.status(request).await?;
        Ok(ProjectorStatus::from(response.into_inner()))
    }
}
