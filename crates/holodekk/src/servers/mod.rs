mod projector;
pub use projector::*;

use std::net::SocketAddr;

use futures_util::FutureExt;
use tokio::{net::UnixListener, sync::oneshot::channel};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::server::TcpIncoming;

use crate::utils::{fs::cleanup, ConnectionInfo};

pub struct GrpcServerHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    task_handle: tokio::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>,
}

impl GrpcServerHandle {
    pub fn new(
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
        task_handle: tokio::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>,
    ) -> Self {
        Self {
            shutdown_tx,
            task_handle,
        }
    }

    pub async fn stop(self) -> std::result::Result<(), tonic::transport::Error> {
        self.shutdown_tx.send(()).unwrap();
        self.task_handle.await.unwrap()
    }
}

pub fn start_grpc_server(
    listener_config: &ConnectionInfo,
    server: tonic::transport::server::Router,
) -> GrpcServerHandle {
    let (shutdown_tx, shutdown_rx) = channel();
    let task_handle = match listener_config {
        ConnectionInfo::Tcp { port, addr } => {
            let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();
            let listener = TcpIncoming::new(listen_address, true, None).unwrap();
            tokio::spawn(async {
                server
                    .serve_with_incoming_shutdown(listener, shutdown_rx.map(drop))
                    .await
            })
        }
        ConnectionInfo::Unix { socket } => {
            cleanup(&socket).unwrap();
            let uds = UnixListener::bind(socket).unwrap();
            let listener = UnixListenerStream::new(uds);
            tokio::spawn(async {
                server
                    .serve_with_incoming_shutdown(listener, shutdown_rx.map(drop))
                    .await
            })
        }
    };

    GrpcServerHandle::new(shutdown_tx, task_handle)
}
