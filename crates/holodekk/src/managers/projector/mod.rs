mod shutdown;
pub use shutdown::{shutdown_projector, ShutdownError};
mod spawn;
pub use spawn::{spawn_projector, SpawnError};

use std::sync::Arc;

use crate::config::HolodekkConfig;
use crate::core::entities::Projector;
use crate::managers::{start_manager, ManagerHandle};

#[derive(Debug)]
pub enum ProjectorCommand {
    Spawn {
        namespace: String,
        resp: tokio::sync::oneshot::Sender<std::result::Result<Projector, SpawnError>>,
    },
    Shutdown {
        projector: Projector,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), ShutdownError>>,
    },
}

#[derive(Debug)]
pub struct ProjectorManager {
    cmd_tx: tokio::sync::mpsc::Sender<ProjectorCommand>,
    handle: ManagerHandle,
}

impl ProjectorManager {
    fn new(cmd_tx: tokio::sync::mpsc::Sender<ProjectorCommand>, handle: ManagerHandle) -> Self {
        Self { cmd_tx, handle }
    }

    pub fn start(config: Arc<HolodekkConfig>) -> Self {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
        let handle = start_manager(projector_manager(config, cmd_rx));
        Self::new(cmd_tx, handle)
    }

    pub async fn stop(self) {
        self.handle.stop().await;
    }

    pub fn cmd_tx(&self) -> tokio::sync::mpsc::Sender<ProjectorCommand> {
        self.cmd_tx.clone()
    }
}

pub async fn projector_manager(
    config: Arc<HolodekkConfig>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<ProjectorCommand>,
) {
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            ProjectorCommand::Spawn { namespace, resp } => {
                println!("spawning {}", namespace);
                let projector = spawn_projector(config.clone(), &namespace).unwrap();
                resp.send(Ok(projector)).unwrap();
            }
            ProjectorCommand::Shutdown { projector, resp } => {
                shutdown_projector(config.clone(), projector.clone()).unwrap();
                resp.send(Ok(())).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn respond_to_spawn() {
        let config = HolodekkConfig {
            fleet: "test".into(),
            root_path: "/tmp".into(),
            bin_path: "/tmp".into(),
        };
        let manager = ProjectorManager::start(Arc::new(config));
        manager.stop().await;
    }
}
