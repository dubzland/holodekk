use tonic::transport::Channel;

use crate::core::subroutines::entities::Subroutine;
use crate::errors::grpc::GrpcClientResult;

use super::proto::entities::RpcCreateSubroutineRequest;
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

    pub async fn create(
        &self,
        fleet: &str,
        namespace: &str,
        subroutine_definition_id: &str,
    ) -> GrpcClientResult<Subroutine> {
        let req = RpcCreateSubroutineRequest {
            fleet: fleet.into(),
            namespace: namespace.into(),
            subroutine_definition_id: subroutine_definition_id.into(),
        };
        let mut client = self.inner.clone();
        let response = client.create(tonic::Request::new(req)).await?;
        Ok(response.into_inner().into())
    }
}
