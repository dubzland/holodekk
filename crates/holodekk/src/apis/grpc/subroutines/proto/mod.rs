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

use crate::entities::{Subroutine, SubroutineInstance, SubroutineKind, SubroutineStatus};
use entities::{RpcSubroutine, RpcSubroutineInstance, RpcSubroutineKind};

impl From<SubroutineKind> for RpcSubroutineKind {
    fn from(kind: SubroutineKind) -> Self {
        match kind {
            SubroutineKind::Ruby => RpcSubroutineKind::Ruby,
            SubroutineKind::Unknown => RpcSubroutineKind::UnknownSubroutineKind,
        }
    }
}

impl From<Subroutine> for RpcSubroutine {
    fn from(subroutine: Subroutine) -> Self {
        let mut res = Self {
            id: subroutine.id.clone(),
            name: subroutine.name.clone(),
            path: subroutine.path.as_os_str().to_str().unwrap().to_owned(),
            kind: 0,
            instances: vec![],
        };

        match subroutine.kind {
            SubroutineKind::Ruby => res.set_kind(RpcSubroutineKind::Ruby),
            SubroutineKind::Unknown => res.set_kind(RpcSubroutineKind::UnknownSubroutineKind),
        };

        if let Some(instances) = subroutine.instances {
            res.instances = instances.into_iter().map(|i| i.into()).collect();
        }
        res
    }
}

impl From<RpcSubroutine> for Subroutine {
    fn from(subroutine: RpcSubroutine) -> Self {
        let kind = match RpcSubroutineKind::from_i32(subroutine.kind) {
            Some(RpcSubroutineKind::Ruby) => SubroutineKind::Ruby,
            Some(RpcSubroutineKind::UnknownSubroutineKind) => SubroutineKind::Unknown,
            None => SubroutineKind::Unknown,
        };
        let mut res = Self {
            id: subroutine.id,
            name: subroutine.name,
            path: subroutine.path.into(),
            kind,
            instances: None,
        };
        if subroutine.instances.is_empty() {
            res.instances = Some(subroutine.instances.into_iter().map(|i| i.into()).collect());
        }
        res
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
