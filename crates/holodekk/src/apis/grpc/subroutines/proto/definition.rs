use crate::core::entities::{SubroutineDefinition, SubroutineKind};
use crate::core::repositories::RepositoryId;

use super::entities::{RpcSubroutineDefinition, RpcSubroutineKind};

impl From<SubroutineDefinition> for RpcSubroutineDefinition {
    fn from(definition: SubroutineDefinition) -> Self {
        let mut res = Self {
            id: definition.id(),
            name: definition.name.clone(),
            path: definition.path.as_os_str().to_str().unwrap().to_owned(),
            kind: 0,
        };

        match definition.kind {
            SubroutineKind::Ruby => res.set_kind(RpcSubroutineKind::Ruby),
            SubroutineKind::Unknown => res.set_kind(RpcSubroutineKind::UnknownSubroutineKind),
        };

        res
    }
}

impl From<RpcSubroutineDefinition> for SubroutineDefinition {
    fn from(definition: RpcSubroutineDefinition) -> Self {
        let kind = match RpcSubroutineKind::from_i32(definition.kind) {
            Some(RpcSubroutineKind::Ruby) => SubroutineKind::Ruby,
            Some(RpcSubroutineKind::UnknownSubroutineKind) => SubroutineKind::Unknown,
            None => SubroutineKind::Unknown,
        };
        Self {
            name: definition.name,
            path: definition.path.into(),
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
    fn converts_to_subroutine_definition_from_rpc_subroutine_definition() {
        let rpc_definition = RpcSubroutineDefinition {
            id: "abc123".to_owned(),
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: RpcSubroutineKind::Ruby as i32,
        };

        let definition: SubroutineDefinition = rpc_definition.into();

        assert_eq!(definition.name, "test");
        assert_eq!(definition.path, PathBuf::from("/tmp"));
        assert_eq!(definition.kind, SubroutineKind::Ruby);
    }

    #[rstest]
    #[test]
    fn converts_to_rpc_subroutine_definition_from_subroutine_definition() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let definition = SubroutineDefinition {
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: SubroutineKind::Ruby,
        };
        let id = definition.id();
        let rpc_definition: RpcSubroutineDefinition = definition.into();

        assert_eq!(rpc_definition.id, id);
        assert_eq!(rpc_definition.name, "test");
        assert_eq!(rpc_definition.path, "/tmp");
        assert_eq!(rpc_definition.kind, RpcSubroutineKind::Ruby as i32);
    }
}
