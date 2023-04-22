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

use std::path::PathBuf;

use holodekk::core::subroutines::entities::{SubroutineEntity, SubroutineStatus};

impl From<entities::RpcSubroutine> for SubroutineEntity {
    fn from(subroutine: entities::RpcSubroutine) -> Self {
        let status: SubroutineStatus = if let Some(rpc_status) = subroutine.status {
            rpc_status.into()
        } else {
            SubroutineStatus::Unknown
        };

        let mut sub = Self::new(
            subroutine.id,
            PathBuf::from(subroutine.path),
            PathBuf::from(subroutine.pidfile),
            PathBuf::from(subroutine.logfile),
            PathBuf::from(subroutine.log_socket),
            SubroutineStatus::Unknown,
            subroutine.projector_id,
            subroutine.subroutine_definition_id,
        );
        sub.set_status(status);
        sub
    }
}

impl From<SubroutineEntity> for entities::RpcSubroutine {
    fn from(subroutine: SubroutineEntity) -> Self {
        Self {
            id: subroutine.id().into(),
            path: subroutine
                .path()
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
            status: Some(subroutine.status().into()),
            pidfile: subroutine
                .pidfile()
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
            logfile: subroutine
                .logfile()
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
            log_socket: subroutine
                .log_socket()
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
            projector_id: subroutine.projector_id().into(),
            subroutine_definition_id: subroutine.subroutine_definition_id().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::*;

    use holodekk::core::subroutines::entities::{SubroutineEntity, SubroutineStatus};

    use crate::api::fixtures::subroutine;
    use crate::api::subroutines::proto::{
        entities::{RpcSubroutine, RpcSubroutineStatus},
        enums::RpcSubroutineStatusCode,
    };

    #[test]
    fn converts_to_subroutine_from_rpc_subroutine() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let rpc_subroutine = RpcSubroutine {
            id: "subroutine_id".into(),
            path: "/tmp/subroutine".into(),
            status: Some(status),
            pidfile: "/tmp/subroutine/pidfile".into(),
            logfile: "/tmp/subroutine/logfile".into(),
            log_socket: "/tmp/subroutine/log.sock".into(),
            projector_id: "projector_id".into(),
            subroutine_definition_id: "subroutine_definition_id".into(),
        };

        let subroutine: SubroutineEntity = rpc_subroutine.into();

        assert_eq!(subroutine.id(), "subroutine_id");
        assert_eq!(subroutine.path(), &PathBuf::from("/tmp/subroutine"));
        assert!(matches!(subroutine.status(), SubroutineStatus::Stopped));
        assert_eq!(
            subroutine.pidfile(),
            &PathBuf::from("/tmp/subroutine/pidfile")
        );
        assert_eq!(
            subroutine.logfile(),
            &PathBuf::from("/tmp/subroutine/logfile")
        );
        assert_eq!(
            subroutine.log_socket(),
            &PathBuf::from("/tmp/subroutine/log.sock")
        );
        assert_eq!(subroutine.projector_id(), "projector_id");
        assert_eq!(
            subroutine.subroutine_definition_id(),
            "subroutine_definition_id"
        );
    }

    #[rstest]
    #[test]
    fn converts_to_rpc_subroutine_from_subroutine(mut subroutine: SubroutineEntity) {
        subroutine.set_status(SubroutineStatus::Stopped);

        let rpc_subroutine: RpcSubroutine = subroutine.clone().into();

        assert_eq!(rpc_subroutine.id, subroutine.id());
        assert_eq!(&PathBuf::from(rpc_subroutine.path), subroutine.path());
        assert_eq!(&PathBuf::from(rpc_subroutine.pidfile), subroutine.pidfile());
        assert_eq!(&PathBuf::from(rpc_subroutine.logfile), subroutine.logfile());
        assert_eq!(
            &PathBuf::from(rpc_subroutine.log_socket),
            subroutine.log_socket()
        );
        assert_eq!(
            rpc_subroutine.status,
            Some(RpcSubroutineStatus {
                code: RpcSubroutineStatusCode::Stopped as i32,
                pid: None
            })
        );
        assert_eq!(rpc_subroutine.projector_id, subroutine.projector_id());
        assert_eq!(
            rpc_subroutine.subroutine_definition_id,
            subroutine.subroutine_definition_id()
        );
    }
}
