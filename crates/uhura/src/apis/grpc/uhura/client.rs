use tonic::transport::Channel;

use holodekk::errors::grpc::ClientResult;

use crate::entities::UhuraStatus;

use super::proto::{entities::RpcUhuraStatusRequest, RpcUhuraClient};

#[derive(Clone, Debug)]
pub struct Client {
    inner: RpcUhuraClient<Channel>,
}

impl Client {
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcUhuraClient::new(channel),
        }
    }

    /// # Errors
    ///
    /// - Failure to connect to server
    pub async fn status(&self) -> ClientResult<UhuraStatus> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcUhuraStatusRequest {});
        let response = client.status(request).await?;
        Ok(UhuraStatus::from(response.into_inner()))
    }
}
