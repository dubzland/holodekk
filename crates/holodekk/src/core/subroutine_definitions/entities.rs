use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::repositories::subroutine_definition_repo_id;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineKind {
    Unknown,
    Ruby,
}

impl SubroutineKind {
    pub fn detect<P: AsRef<Path>>(path: P) -> SubroutineKind {
        let mut ruby_path = PathBuf::from(path.as_ref());
        ruby_path.push("holodekk.rb");
        if ruby_path.try_exists().unwrap() {
            Self::Ruby
        } else {
            Self::Unknown
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineDefinitionEntity {
    id: String,
    name: String,
    path: PathBuf,
    kind: SubroutineKind,
}

impl SubroutineDefinitionEntity {
    pub fn new<S, P>(name: S, path: P, kind: SubroutineKind) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            id: subroutine_definition_repo_id(name.as_ref()),
            name: name.into(),
            path: path.into(),
            kind,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn kind(&self) -> SubroutineKind {
        self.kind
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
