pub mod definition;
pub use definition::SubroutineDefinition;

pub mod manifest;
pub use manifest::SubroutineManifest;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineKind {
    Unknown,
    Ruby,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Subroutine {
    pub fleet: String,
    pub namespace: String,
    pub path: PathBuf,
    pub status: SubroutineStatus,
    pub subroutine_definition_id: String,
}

impl Subroutine {
    pub fn new<S, P>(fleet: S, namespace: S, path: P, subroutine_definition_id: S) -> Self
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            fleet: fleet.into(),
            namespace: namespace.into(),
            path: path.into(),
            status: SubroutineStatus::Unknown,
            subroutine_definition_id: subroutine_definition_id.into(),
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::config::{
        fixtures::{mock_config, MockConfig},
        HolodekkConfig, ProjectorConfig,
    };
    use crate::core::entities::{
        subroutine::definition::fixtures::subroutine_definition, subroutine::SubroutineDefinition,
    };
    use crate::core::repositories::RepositoryId;

    use super::*;

    #[fixture]
    pub(crate) fn subroutine(
        mock_config: MockConfig,
        subroutine_definition: SubroutineDefinition,
    ) -> Subroutine {
        Subroutine::new(
            mock_config.fleet(),
            mock_config.namespace(),
            "/tmp/holodekk/projector/local/subroutines/test/sub",
            &subroutine_definition.id(),
        )
    }
}
