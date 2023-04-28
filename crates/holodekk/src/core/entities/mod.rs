// mod projector;
// pub use projector::*;
mod scene;
pub use scene::*;
mod subroutine;
pub use subroutine::*;
mod subroutine_definition;
pub use subroutine_definition::*;

// pub mod subroutine_manifest;
// pub use subroutine_manifest::*;

use std::convert::TryFrom;
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use rand::RngCore;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub fn generate_id() -> String {
    let mut bytes: [u8; 32] = [0; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

lazy_static! {
    static ref ENTITY_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum EntityIdError {
    #[error("Invalid EntityId format")]
    Format(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct EntityId(String);

impl EntityId {
    pub fn generate() -> Self {
        Self(generate_id())
    }
}

impl FromStr for EntityId {
    type Err = EntityIdError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if ENTITY_ID_RE.is_match(s) {
            Ok(EntityId(s.to_string()))
        } else {
            Err(EntityIdError::Format(s.to_string()))
        }
    }
}

impl TryFrom<String> for EntityId {
    type Error = EntityIdError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl AsRef<Path> for EntityId {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for EntityId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::core::enums::SubroutineKind;

    use super::*;

    #[fixture]
    pub fn mock_scene() -> SceneEntity {
        SceneEntity::new("test".into())
    }

    #[fixture]
    pub fn mock_subroutine_definition() -> SubroutineDefinitionEntity {
        SubroutineDefinitionEntity::new(
            "test/sub",
            "/tmp/holodekk/subroutines/test/sub",
            SubroutineKind::Ruby,
        )
    }

    #[fixture]
    pub fn mock_subroutine(
        mock_scene: SceneEntity,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) -> SubroutineEntity {
        SubroutineEntity::new(&mock_scene.id, &mock_subroutine_definition.id)
    }
}
