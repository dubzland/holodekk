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
        RpcStatusRequest, RpcSubroutine, RpcSubroutineInstance, RpcSubroutineKind,
        RpcSubroutineStatus, RpcSubroutineStatusCode,
    };
}

pub mod enums {}

pub use pb::subroutines::rpc_subroutines_client::RpcSubroutinesClient;
pub use pb::subroutines::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};

mod status;
pub use status::*;
mod subroutine;
pub use subroutine::*;

use crate::entities::{SubroutineInstance, SubroutineKind, SubroutineStatus};
use entities::{RpcSubroutineInstance, RpcSubroutineKind};

impl From<SubroutineKind> for RpcSubroutineKind {
    fn from(kind: SubroutineKind) -> Self {
        match kind {
            SubroutineKind::Ruby => RpcSubroutineKind::Ruby,
            SubroutineKind::Unknown => RpcSubroutineKind::UnknownSubroutineKind,
        }
    }
}

impl From<RpcSubroutineKind> for SubroutineKind {
    fn from(kind: RpcSubroutineKind) -> Self {
        match kind {
            RpcSubroutineKind::Ruby => SubroutineKind::Ruby,
            RpcSubroutineKind::UnknownSubroutineKind => SubroutineKind::Unknown,
        }
    }
}

impl From<RpcSubroutineInstance> for SubroutineInstance {
    fn from(instance: RpcSubroutineInstance) -> Self {
        let status: SubroutineStatus = if let Some(rpc_status) = instance.status {
            rpc_status.into()
        } else {
            SubroutineStatus::Unknown
        };

        Self {
            fleet: instance.fleet,
            namespace: instance.namespace,
            path: instance.path.into(),
            status,
            subroutine_id: instance.subroutine_id,
        }
    }
}

impl From<SubroutineInstance> for RpcSubroutineInstance {
    fn from(instance: SubroutineInstance) -> Self {
        Self {
            fleet: instance.fleet,
            namespace: instance.namespace,
            path: instance.path.into_os_string().into_string().unwrap(),
            status: Some(instance.status.into()),
            subroutine_id: instance.subroutine_id,
        }
    }
}
