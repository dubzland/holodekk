use tonic::transport::Channel;

use crate::entities::SubroutineStatus;
use crate::errors::grpc::GrpcClientResult;

use super::proto::entities::RpcStatusRequest;
use super::proto::RpcSubroutinesClient;

#[derive(Clone, Debug)]
pub struct SubroutinesApiClient {
    inner: RpcSubroutinesClient<Channel>,
}

impl SubroutinesApiClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcSubroutinesClient::new(channel),
        }
    }

    pub async fn status(&self, name: &str) -> GrpcClientResult<SubroutineStatus> {
        let req = tonic::Request::new(RpcStatusRequest {
            name: name.to_string(),
        });
        let mut client = self.inner.clone();
        let response = client.status(req).await?;
        Ok(response.into_inner().into())
    }
}
