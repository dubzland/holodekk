use super::ProjectorServer;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

pub enum ProjectorCommand {
    Stop {
        completion: Option<tokio::sync::oneshot::Sender<()>>,
    },
}

pub struct ProjectorHandle {
    server: ProjectorServer,
    port: u16,
    cmd_tx: UnboundedSender<ProjectorCommand>,
}

impl ProjectorHandle {
    pub fn new(
        server: ProjectorServer,
        port: u16,
        cmd_tx: UnboundedSender<ProjectorCommand>,
    ) -> Self {
        Self {
            server,
            port,
            cmd_tx,
        }
    }

    pub fn port(&self) -> &u16 {
        &self.port
    }

    pub fn stop(&self) {
        let (status_tx, status_rx) = oneshot::channel();

        let _ = self.cmd_tx.send(ProjectorCommand::Stop {
            completion: Some(status_tx),
        });

        self.server.runtime().block_on(async move {
            let _ = status_rx.await;
        });
    }
}
