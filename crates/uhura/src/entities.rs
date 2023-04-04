use crate::api::proto::entities::{RpcProjectorStatus, RpcSubroutine};

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

#[derive(Clone, Debug)]
pub struct Subroutine {
    pub name: String,
    pub pid: u32,
}

impl From<RpcSubroutine> for Subroutine {
    fn from(subroutine: RpcSubroutine) -> Self {
        Self {
            name: subroutine.name,
            pid: subroutine.pid as u32,
        }
    }
}
