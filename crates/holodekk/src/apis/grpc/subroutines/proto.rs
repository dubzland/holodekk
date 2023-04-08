mod pb {
    pub(crate) mod common {
        tonic::include_proto!("common");
    }

    pub(crate) mod subroutines {
        tonic::include_proto!("subroutines");
    }
}

pub mod entities {
    pub use super::pb::common::RpcEmpty;
    pub use super::pb::subroutines::{
        RpcStatusRequest, RpcSubroutine, RpcSubroutineStatus, RpcSubroutineStatusCode,
    };
}

pub mod enums {}

pub use pb::subroutines::rpc_subroutines_client::RpcSubroutinesClient;
pub use pb::subroutines::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};

use crate::entities::{Subroutine, SubroutineStatus};
use entities::{RpcSubroutine, RpcSubroutineStatus, RpcSubroutineStatusCode};

impl From<SubroutineStatus> for RpcSubroutineStatus {
    fn from(status: SubroutineStatus) -> Self {
        let mut rpc_status = RpcSubroutineStatus::default();

        match status {
            SubroutineStatus::Unknown => {
                rpc_status.set_code(RpcSubroutineStatusCode::UnknownSubroutineStatus);
            }
            SubroutineStatus::Stopped => {
                rpc_status.set_code(RpcSubroutineStatusCode::Stopped);
            }
            SubroutineStatus::Running(pid) => {
                rpc_status.set_code(RpcSubroutineStatusCode::Running);
                rpc_status.pid = Some(pid as i32);
            }
            SubroutineStatus::Crashed => {
                rpc_status.set_code(RpcSubroutineStatusCode::Crashed);
            }
        }
        rpc_status
    }
}

impl From<RpcSubroutineStatus> for SubroutineStatus {
    fn from(response: RpcSubroutineStatus) -> Self {
        match RpcSubroutineStatusCode::from_i32(response.code) {
            Some(RpcSubroutineStatusCode::Stopped) => SubroutineStatus::Stopped,
            Some(RpcSubroutineStatusCode::Running) => {
                SubroutineStatus::Running(response.pid.unwrap() as u32)
            }
            Some(RpcSubroutineStatusCode::Crashed) => SubroutineStatus::Crashed,
            Some(RpcSubroutineStatusCode::UnknownSubroutineStatus) => SubroutineStatus::Unknown,
            None => SubroutineStatus::Unknown,
        }
    }
}

impl From<RpcSubroutine> for Subroutine {
    fn from(rpc_subroutine: RpcSubroutine) -> Self {
        let status: SubroutineStatus = if let Some(rpc_status) = rpc_subroutine.status {
            rpc_status.into()
        } else {
            SubroutineStatus::Unknown
        };

        Self {
            fleet: rpc_subroutine.fleet,
            namespace: rpc_subroutine.namespace,
            name: rpc_subroutine.name,
            path: rpc_subroutine.path.into(),
            status,
        }
    }
}

impl From<Subroutine> for RpcSubroutine {
    fn from(subroutine: Subroutine) -> Self {
        Self {
            fleet: subroutine.fleet,
            namespace: subroutine.namespace,
            name: subroutine.name,
            path: subroutine.path.into_os_string().into_string().unwrap(),
            status: Some(subroutine.status.into()),
        }
    }
}
