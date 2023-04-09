mod instance;
pub use instance::*;
mod manifest;
pub use manifest::*;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn generate_id<S: AsRef<str>>(name: S) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_ref());
    format!("{:x}", hasher.finalize())
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineKind {
    Unknown,
    Ruby,
}

/// A subroutine running somewhere on the Holodekk.
#[derive(Clone, Debug, PartialEq)]
pub struct Subroutine {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
    pub instances: Option<Vec<SubroutineInstance>>,
}

impl Subroutine {
    pub fn new<S, P>(name: S, path: P, kind: SubroutineKind) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: Into<PathBuf>,
    {
        let id = generate_id(name.as_ref());
        Self {
            id,
            name: name.into(),
            path: path.into(),
            kind,
            instances: None,
        }
    }
}
