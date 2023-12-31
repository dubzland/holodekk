use crate::apis::grpc::uhura::proto::entities::RpcUhuraStatus;

#[derive(Clone, Copy, Debug)]
pub struct UhuraStatus {
    pub pid: u32,
}

impl From<RpcUhuraStatus> for UhuraStatus {
    fn from(status: RpcUhuraStatus) -> Self {
        Self {
            pid: status.pid as u32,
        }
    }
}

impl From<UhuraStatus> for RpcUhuraStatus {
    fn from(status: UhuraStatus) -> Self {
        Self {
            pid: status.pid as i32,
        }
    }
}
