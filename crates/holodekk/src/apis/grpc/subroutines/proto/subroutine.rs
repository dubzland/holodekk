use crate::core::entities::{Subroutine, SubroutineKind};

use super::entities::{RpcSubroutine, RpcSubroutineKind};

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
        if !subroutine.instances.is_empty() {
            res.instances = Some(subroutine.instances.into_iter().map(|i| i.into()).collect());
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::*;

    use crate::core::entities::{SubroutineInstance, SubroutineStatus};

    use super::*;
    use crate::apis::grpc::subroutines::proto::entities::{
        RpcSubroutineInstance, RpcSubroutineStatus, RpcSubroutineStatusCode,
    };

    #[test]
    fn converts_to_subroutine_from_rpc_subroutine() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let rpc_subroutine_instance = RpcSubroutineInstance {
            fleet: "test".into(),
            namespace: "test".into(),
            path: "/tmp".into(),
            status: Some(status),
            subroutine_id: "abc123".into(),
        };

        let rpc_subroutine = RpcSubroutine {
            id: "abc123".to_owned(),
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: RpcSubroutineKind::Ruby as i32,
            instances: vec![rpc_subroutine_instance.clone()],
        };

        let mut subroutine: Subroutine = rpc_subroutine.into();

        assert_eq!(subroutine.id, "abc123");
        assert_eq!(subroutine.name, "test");
        assert_eq!(subroutine.path, PathBuf::from("/tmp"));
        assert_eq!(subroutine.kind, SubroutineKind::Ruby);

        assert!(subroutine.instances.is_some());

        let instances = subroutine.instances.take().unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(
            instances[0],
            SubroutineInstance::from(rpc_subroutine_instance)
        );
    }

    #[rstest]
    #[test]
    fn converts_to_rpc_subroutine_from_subroutine() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let instance = SubroutineInstance {
            fleet: "test".into(),
            namespace: "test".into(),
            path: "/tmp".into(),
            status: SubroutineStatus::Stopped,
            subroutine_id: "abc123".into(),
        };

        let subroutine = Subroutine {
            id: "abc123".to_owned(),
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: SubroutineKind::Ruby,
            instances: Some(vec![instance.clone()]),
        };
        let rpc_subroutine: RpcSubroutine = subroutine.into();

        assert_eq!(rpc_subroutine.id, "abc123");
        assert_eq!(rpc_subroutine.name, "test");
        assert_eq!(rpc_subroutine.path, "/tmp");
        assert_eq!(rpc_subroutine.kind, RpcSubroutineKind::Ruby as i32);

        let instances = rpc_subroutine.instances;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0], RpcSubroutineInstance::from(instance));
    }
}
