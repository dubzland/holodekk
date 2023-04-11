use log::debug;

use crate::managers::{start_manager, ManagerHandle};

#[derive(Debug)]
pub enum ProjectorCommand {
    Spawn {
        name: String,
        resp: tokio::sync::oneshot::Sender<String>,
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

    pub fn start() -> Self {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
        let handle = start_manager(cmd_rx, projector_manager);
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
    mut cmd_rx: tokio::sync::mpsc::Receiver<ProjectorCommand>,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    tokio::select! {
        _ = async {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    ProjectorCommand::Spawn { name, resp } => {
                        println!("spawning {}", name);
                        resp.send("spawned".to_string()).unwrap();
                    }
                }
            }
        } => {}
        _ = shutdown_rx => {
            debug!("projector shutdown received");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn respond_to_spawn() {
        let manager = ProjectorManager::start();
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let cmd = ProjectorCommand::Spawn {
            name: "test".into(),
            resp: resp_tx,
        };
        manager.cmd_tx().send(cmd).await.unwrap();
        let res: String = resp_rx.await.unwrap();
        manager.stop().await;

        assert_eq!(res, "spawned");
    }
}
