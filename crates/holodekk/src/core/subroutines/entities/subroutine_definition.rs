use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineKind {
    Unknown,
    Ruby,
}

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
