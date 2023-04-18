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
pub struct Subroutine {
    fleet: String,
    namespace: String,
    path: PathBuf,
    status: SubroutineStatus,
    subroutine_definition_id: String,
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

    pub fn fleet(&self) -> &str {
        &self.fleet
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn status(&self) -> SubroutineStatus {
        self.status
    }

    pub fn subroutine_definition_id(&self) -> &str {
        &self.subroutine_definition_id
    }

    pub fn set_status(&mut self, status: SubroutineStatus) {
        self.status = status;
    }
}
