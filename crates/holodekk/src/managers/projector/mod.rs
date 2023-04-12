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

    pub fn start<C>(config: Arc<C>) -> Self
    where
        C: HolodekkConfig + 'static,
    {
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

pub async fn projector_manager<C>(
    config: Arc<C>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<ProjectorCommand>,
) where
    C: HolodekkConfig,
{
    let config = config.clone();
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            ProjectorCommand::Spawn { namespace, resp } => {
                println!("spawning {}", namespace);
                let projector = spawn_projector(config.clone(), &namespace).unwrap();
                resp.send(Ok(projector)).unwrap();
            }
            ProjectorCommand::Shutdown { projector, resp } => {
                shutdown_projector(projector.clone()).unwrap();
                resp.send(Ok(())).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::config::fixtures::mock_config;

    use super::*;

    #[tokio::test]
    async fn respond_to_spawn() {
        let config = mock_config();
        let manager = ProjectorManager::start(Arc::new(config));
        manager.stop().await;
    }
}
