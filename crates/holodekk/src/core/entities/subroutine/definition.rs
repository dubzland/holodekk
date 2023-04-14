use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::SubroutineKind;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineDefinition {
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

impl SubroutineDefinition {
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
    pub(crate) fn subroutine_definition() -> SubroutineDefinition {
        SubroutineDefinition::new(
            "test/sub",
            "/tmp/holodekk/subroutines/test/sub",
            SubroutineKind::Ruby,
        )
    }
}
