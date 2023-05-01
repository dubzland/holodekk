mod pb {
    pub(crate) mod subroutine_definitions {
        tonic::include_proto!("subroutine_definitions");
    }
}

pub mod entities {
    pub use super::pb::subroutine_definitions::{
        RpcCreateSubroutineDefinitionRequest, RpcSubroutineDefinition,
    };
}

pub mod enums {
    pub use super::pb::subroutine_definitions::RpcSubroutineKind;
}

pub use pb::subroutine_definitions::rpc_subroutine_definitions_client::RpcSubroutineDefinitionsClient;
pub use pb::subroutine_definitions::rpc_subroutine_definitions_server::{
    RpcSubroutineDefinitions, RpcSubroutineDefinitionsServer,
};

mod kind;
use holodekk::core::subroutine_definitions::entities::{
    SubroutineDefinitionEntity, SubroutineKind,
};

// use super::entities::{RpcSubroutineDefinition, RpcSubroutineKind};

impl From<SubroutineDefinitionEntity> for entities::RpcSubroutineDefinition {
    fn from(definition: SubroutineDefinitionEntity) -> Self {
        let mut res = Self {
            id: definition.id().into(),
            name: definition.name().into(),
            path: definition.path().as_os_str().to_str().unwrap().to_owned(),
            kind: 0,
        };

        match definition.kind() {
            SubroutineKind::Ruby => res.set_kind(enums::RpcSubroutineKind::Ruby),
            SubroutineKind::Unknown => {
                res.set_kind(enums::RpcSubroutineKind::UnknownSubroutineKind)
            }
        };

        res
    }
}

impl From<entities::RpcSubroutineDefinition> for SubroutineDefinitionEntity {
    fn from(definition: entities::RpcSubroutineDefinition) -> Self {
        let kind = match enums::RpcSubroutineKind::from_i32(definition.kind) {
            Some(enums::RpcSubroutineKind::Ruby) => SubroutineKind::Ruby,
            Some(enums::RpcSubroutineKind::UnknownSubroutineKind) => SubroutineKind::Unknown,
            None => SubroutineKind::Unknown,
        };
        Self::new(definition.name, definition.path, kind)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::*;

    use super::*;
    use crate::api::subroutines::proto::{
        entities::RpcSubroutineStatus, enums::RpcSubroutineStatusCode,
    };

    #[test]
    fn converts_to_subroutine_definition_from_rpc_subroutine_definition() {
        let rpc_definition = entities::RpcSubroutineDefinition {
            id: "abc123".to_owned(),
            name: "test".to_owned(),
            path: "/tmp".into(),
            kind: enums::RpcSubroutineKind::Ruby as i32,
        };

        let definition: SubroutineDefinitionEntity = rpc_definition.into();

        assert_eq!(definition.name(), "test");
        assert_eq!(definition.path(), &PathBuf::from("/tmp"));
        assert_eq!(definition.kind(), SubroutineKind::Ruby);
    }

    #[rstest]
    #[test]
    fn converts_to_rpc_subroutine_definition_from_subroutine_definition() {
        let mut status = RpcSubroutineStatus::default();
        status.set_code(RpcSubroutineStatusCode::Stopped);

        let definition = SubroutineDefinitionEntity::new("test", "/tmp", SubroutineKind::Ruby);
        let id = definition.id().to_owned();
        let rpc_definition = entities::RpcSubroutineDefinition::from(definition);

        assert_eq!(rpc_definition.id, id);
        assert_eq!(rpc_definition.name, "test");
        assert_eq!(rpc_definition.path, "/tmp");
        assert_eq!(rpc_definition.kind, enums::RpcSubroutineKind::Ruby as i32);
    }
}
