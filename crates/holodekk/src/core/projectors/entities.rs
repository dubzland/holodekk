use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::utils::generate_id;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ProjectorStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectorEntity {
    id: String,
    namespace: String,
    root: PathBuf,
    pidfile: PathBuf,
    uhura_socket: PathBuf,
    projector_socket: PathBuf,
    status: ProjectorStatus,
}

impl ProjectorEntity {
    pub fn new<S, P>(namespace: S, projector_root: P) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: AsRef<Path>,
    {
        let root = projector_root.as_ref().to_owned();
        let mut pidfile = root.clone();
        pidfile.push("uhura.pid");
        let mut uhura_socket = root.clone();
        uhura_socket.push("uhura.sock");
        let mut projector_socket = root.clone();
        projector_socket.push("projector.sock");

        Self {
            id: generate_id(),
            root,
            namespace: namespace.into(),
            pidfile,
            uhura_socket,
            projector_socket,
            status: ProjectorStatus::Unknown,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn uhura_socket(&self) -> &PathBuf {
        &self.uhura_socket
    }

    pub fn projector_socket(&self) -> &PathBuf {
        &self.projector_socket
    }

    pub fn status(&self) -> ProjectorStatus {
        self.status
    }

    pub fn set_status(&mut self, status: ProjectorStatus) {
        self.status = status
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn projector() -> ProjectorEntity {
        ProjectorEntity::new("test", "/tmp/projector")
    }
}
