use tonic::transport::Channel;

use holodekk::errors::grpc::GrpcClientResult;

use crate::entities::UhuraStatus;

use super::proto::{entities::RpcUhuraStatusRequest, RpcUhuraClient};

#[derive(Clone, Debug)]
pub struct ApiClient {
    inner: RpcUhuraClient<Channel>,
}

impl ApiClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcUhuraClient::new(channel),
        }
    }

    pub async fn status(&self) -> GrpcClientResult<UhuraStatus> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcUhuraStatusRequest {});
        let response = client.status(request).await?;
        Ok(UhuraStatus::from(response.into_inner()))
    }
}
