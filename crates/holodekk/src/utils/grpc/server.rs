use std::net::SocketAddr;
use std::sync::RwLock;

use futures_util::FutureExt;
use log::{debug, info, warn};
use tokio::{net::UnixListener, sync::oneshot::channel};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::server::TcpIncoming;

use crate::utils::{fs::cleanup, ConnectionInfo};

pub struct Handle {
    shutdown_tx: RwLock<Option<tokio::sync::oneshot::Sender<()>>>,
    task_handle:
        RwLock<Option<tokio::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>>>,
}

impl Handle {
    pub fn new(
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
        task_handle: tokio::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>,
    ) -> Self {
        Self {
            shutdown_tx: RwLock::new(Some(shutdown_tx)),
            task_handle: RwLock::new(Some(task_handle)),
        }
    }

    pub async fn stop(&self) -> std::result::Result<(), tonic::transport::Error> {
        let shutdown_tx = self.shutdown_tx.write().unwrap().take().unwrap();
        let handle = self.task_handle.write().unwrap().take().unwrap();
        shutdown_tx.send(()).unwrap();
        handle.await.unwrap()
        // self.task_handle.write().unwrap().take()
        // await.unwrap()
    }
}

pub fn start(listener_config: &ConnectionInfo, server: tonic::transport::server::Router) -> Handle {
    let (shutdown_tx, shutdown_rx) = channel();
    let task_handle = match listener_config {
        ConnectionInfo::Tcp { port, addr } => {
            let listen_address: SocketAddr = format!("{addr}:{port}").parse().unwrap();
            let listener = TcpIncoming::new(listen_address, true, None).unwrap();
            tokio::spawn(async {
                server
                    .serve_with_incoming_shutdown(listener, shutdown_rx.map(drop))
                    .await
            })
        }
        ConnectionInfo::Unix { socket } => {
            cleanup(socket).unwrap();
            info!("setting up listener at {}", socket.display());
            info!(
                "checking for existence of {}",
                socket.parent().unwrap().display()
            );
            match socket.parent().unwrap().try_exists() {
                Ok(exists) => {
                    if exists {
                        info!("socket directory exists");
                    } else {
                        warn!("socket directory DOES NOT exist");
                    }
                }
                Err(err) => {
                    warn!("Error checking for directory existence: {}", err);
                }
            }
            let uds = UnixListener::bind(socket).unwrap();
            let listener = UnixListenerStream::new(uds);
            tokio::spawn(async {
                debug!("inside spawned UNIX listener");
                server
                    .serve_with_incoming_shutdown(listener, shutdown_rx.map(drop))
                    .await
            })
        }
    };

    Handle::new(shutdown_tx, task_handle)
}
