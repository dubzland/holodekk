pub mod projector;

use std::future::Future;

use log::debug;

#[derive(Debug)]
pub struct ManagerHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl ManagerHandle {
    pub fn new(
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
        task_handle: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            shutdown_tx,
            task_handle,
        }
    }

    pub async fn stop(self) {
        self.shutdown_tx.send(()).unwrap();
        self.task_handle.await.unwrap()
    }
}

pub fn start_manager<F>(manager_fn: F) -> ManagerHandle
where
    F: Future<Output = ()> + Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let task_handle = tokio::spawn(async move {
        tokio::select! {
            _ = async {
                manager_fn.await
            } => {}
            _ = shutdown_rx => {
                debug!("manager shutdown received");
            }
        }
    });
    ManagerHandle::new(shutdown_tx, task_handle)
}
