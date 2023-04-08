use tonic::transport::Channel;

use crate::entities::UhuraStatus;
use crate::errors::grpc::GrpcClientResult;

use super::proto::entities::RpcEmpty;
use super::proto::RpcUhuraClient;

#[derive(Clone, Debug)]
pub struct UhuraApiClient {
    inner: RpcUhuraClient<Channel>,
}

impl UhuraApiClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcUhuraClient::new(channel),
        }
    }

    pub async fn status(&self) -> GrpcClientResult<UhuraStatus> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcEmpty {});
        let response = client.status(request).await?;
        Ok(UhuraStatus::from(response.into_inner()))
    }
}
