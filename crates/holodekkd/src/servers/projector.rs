use log::debug;

#[derive(Debug)]
pub enum ProjectorCommand {
    Spawn {
        name: String,
        resp: tokio::sync::oneshot::Sender<String>,
    },
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
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn(async { projector_manager(cmd_rx, shutdown_rx).await });
        let cmd = ProjectorCommand::Spawn {
            name: "test".into(),
            resp: resp_tx,
        };
        cmd_tx.send(cmd).await.unwrap();
        let res: String = resp_rx.await.unwrap();
        shutdown_tx.send(()).unwrap();
        handle.await.unwrap();

        assert_eq!(res, "spawned");
    }
}
