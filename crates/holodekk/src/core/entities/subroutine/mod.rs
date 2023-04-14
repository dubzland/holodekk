pub mod instance;
pub use instance::{SubroutineInstance, SubroutineStatus};

pub mod manifest;
pub use manifest::SubroutineManifest;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineKind {
    Unknown,
    Ruby,
}

/// A subroutine running somewhere on the Holodekk.
#[derive(Clone, Debug, PartialEq)]
pub struct Subroutine {
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

impl Subroutine {
    pub fn new<S, P>(name: S, path: P, kind: SubroutineKind) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            name: name.into(),
            path: path.into(),
            kind,
        }
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub(crate) fn subroutine() -> Subroutine {
        Subroutine::new(
            "test/sub",
            "/tmp/holodekk/subroutines/test/sub",
            SubroutineKind::Ruby,
        )
    }
}
