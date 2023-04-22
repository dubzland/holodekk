use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::projectors::entities::ProjectorEntity;
use crate::core::subroutine_definitions::entities::SubroutineDefinitionEntity;
use crate::utils::generate_id;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineEntity {
    id: String,
    path: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    log_socket: PathBuf,
    status: SubroutineStatus,
    projector_id: String,
    subroutine_definition_id: String,
}

impl SubroutineEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        path: PathBuf,
        pidfile: PathBuf,
        logfile: PathBuf,
        log_socket: PathBuf,
        status: SubroutineStatus,
        projector_id: String,
        subroutine_definition_id: String,
    ) -> Self {
        Self {
            id,
            path,
            status,
            pidfile,
            logfile,
            log_socket,
            projector_id,
            subroutine_definition_id,
        }
    }

    pub fn build(
        projector: &ProjectorEntity,
        subroutine_definition: &SubroutineDefinitionEntity,
    ) -> Self {
        let id = generate_id();
        let mut path: PathBuf = projector.root().into();
        path.push(id.clone());

        let mut pidfile = path.clone();
        pidfile.push("subroutine.pid");
        let mut logfile = path.clone();
        logfile.push("subroutine.log");
        let mut log_socket = path.clone();
        log_socket.push("log.sock");

        Self::new(
            id,
            path,
            pidfile,
            logfile,
            log_socket,
            SubroutineStatus::Unknown,
            projector.id().into(),
            subroutine_definition.id().into(),
        )
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn status(&self) -> SubroutineStatus {
        self.status
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn logfile(&self) -> &PathBuf {
        &self.logfile
    }

    pub fn log_socket(&self) -> &PathBuf {
        &self.log_socket
    }

    pub fn projector_id(&self) -> &str {
        &self.projector_id
    }

    pub fn subroutine_definition_id(&self) -> &str {
        &self.subroutine_definition_id
    }

    pub fn set_status(&mut self, status: SubroutineStatus) {
        self.status = status;
    }
}
