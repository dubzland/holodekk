//! Helpers for dealing with Grpc (tonic) servers.
use std::net::SocketAddr;

use async_trait::async_trait;
use futures_util::FutureExt;
use log::{info, warn};
use tokio::{
    net::UnixListener,
    sync::oneshot::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::server::TcpIncoming;

use crate::utils::{fs::remove_file, ConnectionInfo};

/// A Handle implementation for an instance of a Grpc server.
pub struct Handle {
    shutdown_tx: Sender<()>,
    task_handle: JoinHandle<std::result::Result<(), tonic::transport::Error>>,
}

impl Handle {
    /// Constructs a new [`Handle`] instance.
    #[must_use]
    pub fn new(
        shutdown_tx: Sender<()>,
        task_handle: JoinHandle<std::result::Result<(), tonic::transport::Error>>,
    ) -> Self {
        Self {
            shutdown_tx,
            task_handle,
        }
    }
}

#[async_trait]
impl super::Handle<tonic::transport::Error> for Handle {
    /// Sends a shutdown signal to the running server, and waits for the task to complete.
    async fn stop(self) -> std::result::Result<(), tonic::transport::Error> {
        self.shutdown_tx.send(()).unwrap();
        self.task_handle.await.unwrap()
    }
}

/// A Grpc server instance.
pub struct Grpc {
    config: ConnectionInfo,
    router: tonic::transport::server::Router,
    shutdown_rx: Receiver<()>,
}

impl Grpc {
    async fn run(self) -> std::result::Result<(), tonic::transport::Error> {
        match self.config {
            ConnectionInfo::Tcp { port, addr } => {
                let listen_address: SocketAddr = format!("{addr}:{port}").parse().unwrap();
                let listener = TcpIncoming::new(listen_address, true, None).unwrap();
                self.router
                    .serve_with_incoming_shutdown(listener, self.shutdown_rx.map(drop))
                    .await
            }
            ConnectionInfo::Unix { socket } => {
                remove_file(&socket).unwrap();
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
                self.router
                    .serve_with_incoming_shutdown(listener, self.shutdown_rx.map(drop))
                    .await
            }
        }
    }
}

impl super::Server<tonic::transport::server::Router, tonic::transport::Error> for Grpc {
    type Handle = Handle;

    /// Starts the given Grpc (tonic) server running with the provided listener configuration.
    fn start(listener_config: &ConnectionInfo, router: tonic::transport::server::Router) -> Handle {
        let (shutdown_tx, shutdown_rx) = channel();
        let config = listener_config.clone();
        let task_handle = tokio::spawn(async move {
            let server = Grpc {
                config,
                router,
                shutdown_rx,
            };
            server.run().await
        });

        Handle::new(shutdown_tx, task_handle)
    }
}
