mod pb {
    pub(crate) mod subroutines {
        tonic::include_proto!("subroutines");
    }
}

pub mod entities {
    pub use super::pb::subroutines::{
        RpcCreateSubroutineRequest, RpcSubroutine, RpcSubroutineStatus,
    };
}

pub mod enums {
    pub use super::pb::subroutines::RpcSubroutineStatusCode;
}

pub use pb::subroutines::rpc_subroutines_client::RpcSubroutinesClient;
pub use pb::subroutines::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};

mod status;

use crate::core::subroutines::entities::{Subroutine, SubroutineStatus};

impl From<entities::RpcSubroutine> for Subroutine {
    fn from(subroutine: entities::RpcSubroutine) -> Self {
        let status: SubroutineStatus = if let Some(rpc_status) = subroutine.status {
            rpc_status.into()
        } else {
            SubroutineStatus::Unknown
        };

        Self {
            fleet: subroutine.fleet,
            namespace: subroutine.namespace,
            path: subroutine.path.into(),
            status,
            subroutine_definition_id: subroutine.subroutine_definition_id,
        }
    }
}

impl From<Subroutine> for entities::RpcSubroutine {
    fn from(subroutine: Subroutine) -> Self {
        Self {
            fleet: subroutine.fleet,
            namespace: subroutine.namespace,
            path: subroutine.path.into_os_string().into_string().unwrap(),
            status: Some(subroutine.status.into()),
            subroutine_definition_id: subroutine.subroutine_definition_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::core::subroutines::api::proto::{
        entities::{RpcSubroutine, RpcSubroutineStatus},
        enums::RpcSubroutineStatusCode,
    };
    use crate::core::subroutines::entities::{Subroutine, SubroutineStatus};

    #[test]
    fn converts_to_subroutine_from_rpc_subroutine() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let rpc_subroutine = RpcSubroutine {
            fleet: "test".into(),
            namespace: "test".into(),
            path: "/tmp".into(),
            status: Some(status),
            subroutine_definition_id: "abc123".into(),
        };

        let subroutine: Subroutine = rpc_subroutine.into();

        assert_eq!(subroutine.fleet, "test");
        assert_eq!(subroutine.namespace, "test");
        assert_eq!(subroutine.path, PathBuf::from("/tmp"));
        assert!(matches!(subroutine.status, SubroutineStatus::Stopped));
        assert_eq!(subroutine.subroutine_definition_id, "abc123");
    }

    #[test]
    fn converts_to_rpc_subroutine_from_subroutine() {
        let subroutine = Subroutine {
            fleet: "test".into(),
            namespace: "test".into(),
            path: "/tmp".into(),
            status: SubroutineStatus::Stopped,
            subroutine_definition_id: "abc123".into(),
        };

        let rpc_subroutine: RpcSubroutine = subroutine.into();

        assert_eq!(rpc_subroutine.fleet, "test");
        assert_eq!(rpc_subroutine.namespace, "test");
        assert_eq!(rpc_subroutine.path, "/tmp");
        assert_eq!(
            rpc_subroutine.status,
            Some(RpcSubroutineStatus {
                code: RpcSubroutineStatusCode::Stopped as i32,
                pid: None
            })
        );
        assert_eq!(rpc_subroutine.subroutine_definition_id, "abc123");
    }
}
