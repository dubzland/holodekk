use log::debug;
use tokio::sync::{
    mpsc::{channel, Sender},
    oneshot::Sender as OneshotSender,
};

#[derive(Debug)]
pub enum ProjectorCommand {
    Spawn {
        name: String,
        resp: OneshotSender<String>,
    },
}

pub struct ProjectorService {
    sender: Sender<ProjectorCommand>,
    shutdown: OneshotSender<()>,
    handle: tokio::task::JoinHandle<std::result::Result<(), std::io::Error>>,
}

impl ProjectorService {
    fn new(
        sender: Sender<ProjectorCommand>,
        shutdown: OneshotSender<()>,
        handle: tokio::task::JoinHandle<std::result::Result<(), std::io::Error>>,
    ) -> Self {
        Self {
            sender,
            shutdown,
            handle,
        }
    }

    pub fn start() -> Self {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let (sender, mut receiver) = channel::<ProjectorCommand>(32);
        let runtime_handle = tokio::runtime::Handle::current();
        let handle = runtime_handle.spawn(async move {
            tokio::select! {
                _ = async {
                    while let Some(cmd) = receiver.recv().await {
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
            Ok(())
        });
        Self::new(sender, shutdown_tx, handle)
    }

    pub async fn stop(self) {
        self.shutdown.send(()).unwrap();
        self.handle.await.unwrap().unwrap();
    }

    pub fn handle(&self) -> Sender<ProjectorCommand> {
        self.sender.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        let service = ProjectorService::start();

        service.stop().await;
    }

    #[tokio::test]
    async fn respond_to_spawn() {
        let service = ProjectorService::start();
        let tx = service.handle();
        let (cmd_tx, cmd_rx) = tokio::sync::oneshot::channel::<String>();
        let cmd = ProjectorCommand::Spawn {
            name: "test".into(),
            resp: cmd_tx,
        };
        tx.send(cmd).await.unwrap();
        let res: String = cmd_rx.await.unwrap();
        service.stop().await;

        assert_eq!(res, "spawned");
    }
}
