use std::process::Command;

use log::{error, trace};

use crate::config::HolodekkConfig;
use crate::core::projectors::entities::ProjectorEntity;
use crate::core::subroutine_definitions::entities::SubroutineDefinitionEntity;
use crate::core::subroutines::entities::{SubroutineEntity, SubroutineStatus};
use crate::process::{daemonize, DaemonizeError};
use crate::utils::fs::ensure_directory;

use super::SubroutinesWorker;

#[derive(thiserror::Error, Debug)]
pub enum SpawnError {
    #[error("failed to setup subroutine root directory")]
    Setup(#[from] std::io::Error),
    #[error("failed to daemonize subroutine process")]
    Daemonize(#[from] DaemonizeError),
}

impl<C> SubroutinesWorker<C>
where
    C: HolodekkConfig,
{
    pub async fn spawn(
        &self,
        projector: &ProjectorEntity,
        definition: &SubroutineDefinitionEntity,
    ) -> std::result::Result<SubroutineEntity, SpawnError> {
        trace!("SubrotinesWorker::spawn({:?}, {:?})", projector, definition);

        let mut subroutine = SubroutineEntity::build(projector, definition);

        let mut root_path = self.config.subroutines_root().clone();
        root_path.push(projector.namespace());

        // ensure the root directory exists
        ensure_directory(subroutine.path())?;

        // build and execute the actual projector command
        let mut subroutine_executable = self.config.bin_root().clone();
        subroutine_executable.push("holodekk-subroutine");

        let mut command = Command::new(subroutine_executable);
        command.arg("--path");
        command.arg(definition.path());
        command.arg("--subroutine");
        command.arg("default");
        command.arg("--projector-socket");
        command.arg(projector.projector_socket());

        let pid = daemonize(self.config.clone(), command, subroutine.pidfile())?;

        subroutine.set_status(SubroutineStatus::Running(pid as u32));
        Ok(subroutine)
    }
}
