use crate::api::proto::entities::RpcProjectorStatus;

#[derive(Clone, Copy, Debug)]
pub struct ProjectorStatus {
    pub pid: u32,
}

impl From<RpcProjectorStatus> for ProjectorStatus {
    fn from(status: RpcProjectorStatus) -> Self {
        Self {
            pid: status.pid as u32,
        }
    }
}

impl From<ProjectorStatus> for RpcProjectorStatus {
    fn from(status: ProjectorStatus) -> Self {
        Self {
            pid: status.pid as i32,
        }
    }
}
