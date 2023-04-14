use std::path::Path;

use tonic::transport::Channel;

use crate::core::entities::{Subroutine, SubroutineKind};
use crate::errors::grpc::GrpcClientResult;

use super::proto::entities::{RpcCreateRequest, RpcSubroutineKind};
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
        name: &str,
        path: &Path,
        kind: SubroutineKind,
    ) -> GrpcClientResult<Subroutine> {
        let mut req = RpcCreateRequest {
            name: name.into(),
            path: path.as_os_str().to_owned().into_string().unwrap(),
            kind: 0,
        };
        req.set_kind(RpcSubroutineKind::from(kind));
        // let mut req = tonic::Request::new(RpcCreateRequest {
        //     name: name.into(),
        //     path: path.into_os_string().into_string().unwrap(),
        //     kind: 0,
        // });
        let mut client = self.inner.clone();
        let response = client.create(tonic::Request::new(req)).await?;
        Ok(response.into_inner().into())
    }
}
