use tonic::transport::Channel;

use crate::entities::ProjectorStatus;

use crate::api::proto::{core::RpcCoreClient, entities::RpcEmpty};

#[derive(Clone, Debug)]
pub struct CoreClient {
    inner: RpcCoreClient<Channel>,
}

impl CoreClient {
    pub fn new(client: RpcCoreClient<Channel>) -> Self {
        Self { inner: client }
    }

    pub async fn status(&self) -> super::Result<ProjectorStatus> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcEmpty {});
        let response = client.status(request).await?;
        Ok(ProjectorStatus::from(response.into_inner()))
    }
}
