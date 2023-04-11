use crate::core::entities::{SubroutineInstance, SubroutineStatus};

use super::entities::RpcSubroutineInstance;

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::apis::grpc::subroutines::proto::entities::{
        RpcSubroutineInstance, RpcSubroutineStatus, RpcSubroutineStatusCode,
    };
    use crate::core::entities::{SubroutineInstance, SubroutineStatus};

    #[test]
    fn converts_to_instance_from_rpc_instance() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let rpc_instance = RpcSubroutineInstance {
            fleet: "test".into(),
            namespace: "test".into(),
            path: "/tmp".into(),
            status: Some(status),
            subroutine_id: "abc123".into(),
        };

        let instance: SubroutineInstance = rpc_instance.into();

        assert_eq!(instance.fleet, "test");
        assert_eq!(instance.namespace, "test");
        assert_eq!(instance.path, PathBuf::from("/tmp"));
        assert!(matches!(instance.status, SubroutineStatus::Stopped));
        assert_eq!(instance.subroutine_id, "abc123");
    }

    #[test]
    fn converts_to_rpc_instance_from_instance() {
        let instance = SubroutineInstance {
            fleet: "test".into(),
            namespace: "test".into(),
            path: "/tmp".into(),
            status: SubroutineStatus::Stopped,
            subroutine_id: "abc123".into(),
        };

        let rpc_instance: RpcSubroutineInstance = instance.into();

        assert_eq!(rpc_instance.fleet, "test");
        assert_eq!(rpc_instance.namespace, "test");
        assert_eq!(rpc_instance.path, "/tmp");
        assert_eq!(
            rpc_instance.status,
            Some(RpcSubroutineStatus {
                code: RpcSubroutineStatusCode::Stopped as i32,
                pid: None
            })
        );
        assert_eq!(rpc_instance.subroutine_id, "abc123");
    }
}
