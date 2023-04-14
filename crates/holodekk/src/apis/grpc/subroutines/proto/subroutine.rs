use crate::core::entities::{Subroutine, SubroutineKind};
use crate::core::repositories::RepositoryId;

use super::entities::{RpcSubroutine, RpcSubroutineKind};

impl From<Subroutine> for RpcSubroutine {
    fn from(subroutine: Subroutine) -> Self {
        let mut res = Self {
            id: subroutine.id(),
            name: subroutine.name.clone(),
            path: subroutine.path.as_os_str().to_str().unwrap().to_owned(),
            kind: 0,
        };

        match subroutine.kind {
            SubroutineKind::Ruby => res.set_kind(RpcSubroutineKind::Ruby),
            SubroutineKind::Unknown => res.set_kind(RpcSubroutineKind::UnknownSubroutineKind),
        };

        //         if let Some(instances) = subroutine.instances {
        //             res.instances = instances.into_iter().map(|i| i.into()).collect();
        //         }
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
        Self {
            name: subroutine.name,
            path: subroutine.path.into(),
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::*;

    use super::*;
    use crate::apis::grpc::subroutines::proto::entities::{
        RpcSubroutineStatus, RpcSubroutineStatusCode,
    };

    #[test]
    fn converts_to_subroutine_from_rpc_subroutine() {
        let rpc_subroutine = RpcSubroutine {
            id: "abc123".to_owned(),
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: RpcSubroutineKind::Ruby as i32,
        };

        let subroutine: Subroutine = rpc_subroutine.into();

        assert_eq!(subroutine.name, "test");
        assert_eq!(subroutine.path, PathBuf::from("/tmp"));
        assert_eq!(subroutine.kind, SubroutineKind::Ruby);
    }

    #[rstest]
    #[test]
    fn converts_to_rpc_subroutine_from_subroutine() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let subroutine = Subroutine {
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: SubroutineKind::Ruby,
        };
        let id = subroutine.id();
        let rpc_subroutine: RpcSubroutine = subroutine.into();

        assert_eq!(rpc_subroutine.id, id);
        assert_eq!(rpc_subroutine.name, "test");
        assert_eq!(rpc_subroutine.path, "/tmp");
        assert_eq!(rpc_subroutine.kind, RpcSubroutineKind::Ruby as i32);
    }
}
