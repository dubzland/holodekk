mod shutdown;
pub use shutdown::{shutdown_subroutine, ShutdownError};
mod spawn;
pub use spawn::{spawn_subroutine, SpawnError};

use std::sync::Arc;

use crate::config::{HolodekkConfig, ProjectorConfig};
use crate::core::entities::{Subroutine, SubroutineInstance};
use crate::managers::{start_manager, ManagerHandle};

#[derive(Debug)]
pub enum SubroutineCommand {
    Spawn {
        namespace: String,
        subroutine: Subroutine,
        resp: tokio::sync::oneshot::Sender<std::result::Result<SubroutineInstance, SpawnError>>,
    },
    Shutdown {
        instance: SubroutineInstance,
        subroutine: Subroutine,
        resp: tokio::sync::oneshot::Sender<std::result::Result<(), ShutdownError>>,
    },
}

#[derive(Debug)]
pub struct SubroutineManager {
    cmd_tx: tokio::sync::mpsc::Sender<SubroutineCommand>,
    handle: ManagerHandle,
}

impl SubroutineManager {
    fn new(cmd_tx: tokio::sync::mpsc::Sender<SubroutineCommand>, handle: ManagerHandle) -> Self {
        Self { cmd_tx, handle }
    }

    pub fn start<C>(config: Arc<C>) -> Self
    where
        C: HolodekkConfig + ProjectorConfig + 'static,
    {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
        let handle = start_manager(subroutine_manager(config, cmd_rx));
        Self::new(cmd_tx, handle)
    }

    pub async fn stop(self) {
        self.handle.stop().await;
    }

    pub fn cmd_tx(&self) -> tokio::sync::mpsc::Sender<SubroutineCommand> {
        self.cmd_tx.clone()
    }
}

pub async fn subroutine_manager<C>(
    config: Arc<C>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<SubroutineCommand>,
) where
    C: HolodekkConfig + ProjectorConfig,
{
    let config = config.clone();
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            SubroutineCommand::Spawn {
                namespace,
                subroutine,
                resp,
            } => {
                println!("spawning {}:{}", namespace, &subroutine.name);
                let subroutine_instance =
                    spawn_subroutine(config.clone(), &namespace, subroutine.clone()).unwrap();
                resp.send(Ok(subroutine_instance)).unwrap();
            }
            SubroutineCommand::Shutdown {
                instance,
                subroutine,
                resp,
            } => {
                shutdown_subroutine(instance.clone(), subroutine.clone()).unwrap();
                resp.send(Ok(())).unwrap();
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use crate::config::fixtures::mock_config;

//     use super::*;

//     #[tokio::test]
//     async fn respond_to_spawn() {
//         let config = mock_config();
//         let manager = ProjectorManager::start(Arc::new(config));
//         manager.stop().await;
//     }
// }
