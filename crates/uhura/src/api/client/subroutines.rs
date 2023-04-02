use tonic::transport::Channel;

use crate::entities::Subroutine;

use crate::api::proto::{entities::RpcEmpty, subroutines::RpcSubroutinesClient};

#[derive(Clone, Debug)]
pub struct SubroutinesClient {
    inner: RpcSubroutinesClient<Channel>,
}

impl SubroutinesClient {
    pub fn new(client: RpcSubroutinesClient<Channel>) -> Self {
        Self { inner: client }
    }

    pub async fn status(&self) -> super::Result<Vec<Subroutine>> {
        let req = tonic::Request::new(RpcEmpty {});
        let mut client = self.inner.clone();
        let response = client.list(req).await?;
        let subroutines: Vec<Subroutine> = response
            .into_inner()
            .subroutines
            .into_iter()
            .map(|s| s.into())
            .collect();
        Ok(subroutines)
    }
}
