pub use futures_core::future::BoxFuture;

use std::fmt;

use async_trait::async_trait;
use log::warn;
use tokio::{sync::oneshot::Sender, task::JoinHandle};

use super::TonicResult;

use crate::server::{ServerHandle, ServerStatus};

type TonicJoinHandle = JoinHandle<TonicResult>;

/// Handle to the currently running Tonic server
pub struct TonicServerHandle {
    cmd_tx: Option<Sender<()>>,
    fut: Option<BoxFuture<'static, TonicResult>>,
    handle: Option<TonicJoinHandle>,
    status: ServerStatus,
}

impl fmt::Debug for TonicServerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TonicServerHandle")
            .field("cmd_tx", &self.cmd_tx)
            .field("handle", &self.handle)
            .field("status", &self.status)
            .finish()
    }
}

impl TonicServerHandle {
    pub fn new(cmd_tx: Sender<()>, fut: BoxFuture<'static, TonicResult>) -> Self {
        Self {
            cmd_tx: Some(cmd_tx),
            fut: Some(fut),
            handle: None,
            status: ServerStatus::Pending,
        }
    }
}

#[async_trait]
impl ServerHandle for TonicServerHandle {
    type Result = ();
    type Error = tonic::transport::Error;

    fn start(&mut self) {
        let runtime_handle = tokio::runtime::Handle::current();
        let fut = self.fut.take().unwrap();
        let handle = runtime_handle.spawn(async { fut.await });
        self.handle.replace(handle);
        self.status = ServerStatus::Running;
    }

    async fn stop(&mut self) -> std::result::Result<Self::Result, Self::Error> {
        self.cmd_tx.take().unwrap().send(()).unwrap();
        self.status = ServerStatus::Stopped;
        match self.handle.take().unwrap().await {
            Ok(res) => res,
            Err(err) => {
                warn!("Error waiting for shutdown response: {}", err);
                Ok(())
            }
        }
    }

    fn status(&self) -> ServerStatus {
        self.status
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::sync::oneshot::{
        channel as tokio_channel, Receiver as TokioReceiver, Sender as TokioSender,
    };

    async fn test_router(cmd_rx: TokioReceiver<()>, data_tx: TokioSender<i32>) -> TonicResult {
        cmd_rx.await.unwrap();
        data_tx.send(42).unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn tonic_server_handle_runs_future() {
        let (cmd_tx, cmd_rx) = tokio_channel::<()>();
        let (data_tx, data_rx) = tokio_channel::<i32>();

        let fut = Box::pin(test_router(cmd_rx, data_tx));

        let mut handle = TonicServerHandle::new(cmd_tx, fut);
        handle.start();
        handle.stop().await.unwrap();
        let answer = data_rx.await.unwrap();
        assert_eq!(answer, 42);
    }
}
