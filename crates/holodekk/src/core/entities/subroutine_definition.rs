use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::enums::SubroutineKind;

use super::EntityId;

pub type SubroutineDefinitionEntityId = EntityId;

pub fn generate_subroutine_definition_id(name: &str) -> SubroutineDefinitionEntityId {
    let mut hasher = Sha256::new();
    hasher.update(name);
    format!("{:x}", hasher.finalize()).try_into().unwrap()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineDefinitionEntity {
    pub id: SubroutineDefinitionEntityId,
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

impl SubroutineDefinitionEntity {
    pub fn new<S, P>(name: S, path: P, kind: SubroutineKind) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            id: generate_subroutine_definition_id(name.as_ref()),
            name: name.into(),
            path: path.into(),
            kind,
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub(crate) fn subroutine_definition() -> SubroutineDefinitionEntity {
        SubroutineDefinitionEntity::new(
            "test/sub",
            "/tmp/holodekk/subroutines/test/sub",
            SubroutineKind::Ruby,
        )
    }
}
