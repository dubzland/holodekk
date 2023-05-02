use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SceneStatus {
    Unknown,
    Created,
    Starting(i32),
    Running(i32),
    Stopped,
    Crashed,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ProjectorStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

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
