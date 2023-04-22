use std::process::Command;

use log::{error, trace};

use crate::config::HolodekkConfig;
use crate::core::projectors::entities::{ProjectorEntity, ProjectorStatus};
use crate::process::{daemonize, DaemonizeError};
use crate::utils::fs::ensure_directory;

use super::ProjectorsWorker;

#[derive(thiserror::Error, Debug)]
pub enum SpawnError {
    #[error("failed to setup projector root directory")]
    Setup(#[from] std::io::Error),
    #[error("failed to daemonize projector process")]
    Daemonize(#[from] DaemonizeError),
}

impl<C> ProjectorsWorker<C>
where
    C: HolodekkConfig,
{
    pub async fn spawn(&self, namespace: &str) -> std::result::Result<ProjectorEntity, SpawnError> {
        trace!("ProjectorsWorker::spawn({})", namespace);

        let mut root_path = self.config.projectors_root().clone();
        root_path.push(namespace);

        // ensure the root directory exists
        ensure_directory(&root_path)?;

        let mut projector = ProjectorEntity::new(namespace, root_path);

        // build and execute the actual projector command
        let mut uhura = self.config.bin_root().clone();
        uhura.push("uhura");

        let mut command = Command::new(uhura);
        command.arg("--namespace");
        command.arg(projector.namespace());

        let pid = daemonize(self.config.clone(), command, projector.pidfile())?;

        projector.set_status(ProjectorStatus::Running(pid as u32));
        Ok(projector)
    }
}
