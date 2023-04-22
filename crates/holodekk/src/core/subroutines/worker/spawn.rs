use std::process::Command;

use log::{error, trace};

use crate::config::HolodekkConfig;
use crate::core::projectors::entities::ProjectorEntity;
use crate::core::subroutine_definitions::entities::SubroutineDefinitionEntity;
use crate::core::subroutines::{
    entities::{SubroutineEntity, SubroutineStatus},
    SubroutinePaths,
};
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

        let paths = SubroutinePaths::build(self.config.clone(), &subroutine);

        // ensure the root directory exists
        ensure_directory(paths.root())?;

        // build and execute the actual projector command
        let mut subroutine_executable = self.config.bin_root().clone();
        subroutine_executable.push("holodekk-subroutine");

        let mut command = Command::new(subroutine_executable);
        command.arg("--id");
        command.arg(subroutine.id());
        command.arg("--path");
        command.arg(definition.path());
        command.arg("--subroutine");
        command.arg("default");
        command.arg("--projector-socket");
        command.arg(projector.projector_socket());

        let pid = daemonize(self.config.clone(), command, paths.pidfile())?;

        subroutine.set_status(SubroutineStatus::Running(pid as u32));
        Ok(subroutine)
    }
}
