use tonic::transport::Channel;

use crate::errors::grpc::GrpcClientResult;

use super::proto::entities::RpcEmpty;
use super::proto::RpcApplicationsClient;

#[derive(Clone, Debug)]
pub struct ApplicationsClient {
    inner: RpcApplicationsClient<Channel>,
}

impl ApplicationsClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcApplicationsClient::new(channel),
        }
    }

    pub async fn list(&self) -> GrpcClientResult<String> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcEmpty {});
        let response = client.list(request).await?;
        Ok(response.into_inner().message)
    }
}
