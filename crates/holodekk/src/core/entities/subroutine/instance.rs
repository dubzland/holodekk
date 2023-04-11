use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineInstance {
    pub fleet: String,
    pub namespace: String,
    pub path: PathBuf,
    pub status: SubroutineStatus,
    pub subroutine_id: String,
}

impl SubroutineInstance {
    pub fn new<S, P>(fleet: S, namespace: S, path: P, subroutine_id: S) -> Self
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            fleet: fleet.into(),
            namespace: namespace.into(),
            path: path.into(),
            status: SubroutineStatus::Unknown,
            subroutine_id: subroutine_id.into(),
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::config::{fixtures::holodekk_config, HolodekkConfig};
    use crate::core::entities::{subroutine::fixtures::subroutine, Subroutine};

    use super::*;

    #[fixture]
    pub(crate) fn subroutine_instance(
        holodekk_config: HolodekkConfig,
        subroutine: Subroutine,
    ) -> SubroutineInstance {
        SubroutineInstance::new(
            &holodekk_config.fleet,
            &"test-namespace".to_string(),
            "/tmp/holodekk/projector/local/subroutines/test/sub",
            &subroutine.id,
        )
    }
}
