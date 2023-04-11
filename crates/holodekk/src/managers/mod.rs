pub mod projector;

use std::future::Future;

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

pub type ManagerFn<R, F> =
    fn(tokio::sync::mpsc::Receiver<R>, tokio::sync::oneshot::Receiver<()>) -> F;

pub fn start_manager<R, F>(
    cmd_rx: tokio::sync::mpsc::Receiver<R>,
    manager_fn: ManagerFn<R, F>,
) -> ManagerHandle
where
    F: Future<Output = ()> + Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let task_handle = tokio::spawn(manager_fn(cmd_rx, shutdown_rx));
    ManagerHandle::new(shutdown_tx, task_handle)
}
