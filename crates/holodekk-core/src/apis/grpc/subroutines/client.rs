use tonic::transport::Channel;

use crate::entities::{Subroutine, SubroutineStatus};
use crate::errors::grpc::GrpcClientResult;

use super::proto::entities::{RpcEmpty, RpcSubroutine, RpcSubroutineStatus};
use super::proto::RpcSubroutinesClient;

impl From<Subroutine> for RpcSubroutine {
    fn from(subroutine: Subroutine) -> Self {
        let mut pid = 0;
        let sub_status = subroutine.status;

        let status = match sub_status {
            SubroutineStatus::Stopped => RpcSubroutineStatus::Stopped,
            SubroutineStatus::Running(rpc_pid) => {
                pid = rpc_pid;
                RpcSubroutineStatus::Running
            }
            SubroutineStatus::Crashed => RpcSubroutineStatus::Stopped,
        };

        Self {
            name: subroutine.name,
            path: subroutine.path.into_os_string().into_string().unwrap(),
            status: status.into(),
            pid: pid as i32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubroutinesClient {
    inner: RpcSubroutinesClient<Channel>,
}

impl SubroutinesClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RpcSubroutinesClient::new(channel),
        }
    }

    pub async fn status(&self) -> GrpcClientResult<Vec<Subroutine>> {
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
