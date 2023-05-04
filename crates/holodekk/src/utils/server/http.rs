//! Logic for managing Http servers.

use std::{
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use axum::extract::connect_info;
use futures::ready;
use futures_util::FutureExt;
use hyper::server::accept::Accept;
use log::trace;
use tokio::{
    net::{unix::UCred, UnixListener, UnixStream},
    sync::oneshot::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tower::BoxError;

use crate::utils::{fs::remove_file, ConnectionInfo};

struct AcceptConnection {
    uds: UnixListener,
}

impl Accept for AcceptConnection {
    type Conn = UnixStream;
    type Error = BoxError;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let (stream, _addr) = ready!(self.uds.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct UdsConnectInfo {
    peer_addr: Arc<tokio::net::unix::SocketAddr>,
    peer_cred: UCred,
}

impl connect_info::Connected<&UnixStream> for UdsConnectInfo {
    fn connect_info(target: &UnixStream) -> Self {
        let peer_addr = target.peer_addr().unwrap();
        let peer_cred = target.peer_cred().unwrap();

        Self {
            peer_addr: Arc::new(peer_addr),
            peer_cred,
        }
    }
}

/// A [`super::Handle`] implementation for an instance of an Http server.
#[derive(Debug)]
pub struct Handle {
    shutdown_tx: Sender<()>,
    task_handle: JoinHandle<std::result::Result<(), hyper::Error>>,
}

impl Handle {
    /// Constructs a new [`Handle`] instance.
    #[must_use]
    pub fn new(
        shutdown_tx: Sender<()>,
        task_handle: JoinHandle<std::result::Result<(), hyper::Error>>,
    ) -> Self {
        Self {
            shutdown_tx,
            task_handle,
        }
    }
}

#[async_trait]
impl super::Handle<hyper::Error> for Handle {
    /// Sends a shutdown signal to the running server, and waits for the task to complete.
    async fn stop(self) -> std::result::Result<(), hyper::Error> {
        self.shutdown_tx.send(()).unwrap();
        self.task_handle.await.unwrap()
    }
}

/// An Http server instance.
pub struct Http {
    config: ConnectionInfo,
    router: axum::Router,
    shutdown_rx: Receiver<()>,
}

impl Http {
    async fn run(self) -> std::result::Result<(), hyper::Error> {
        match self.config {
            ConnectionInfo::Tcp { port, addr } => {
                let listen_address: SocketAddr = format!("{addr}:{port}").parse().unwrap();
                axum::Server::bind(&listen_address)
                    .serve(self.router.into_make_service())
                    .with_graceful_shutdown(self.shutdown_rx.map(drop))
                    .await
            }
            ConnectionInfo::Unix { socket } => {
                remove_file(&socket).unwrap();
                trace!("setting up listener at {}", socket.display());
                let uds = UnixListener::bind(socket).unwrap();
                axum::Server::builder(AcceptConnection { uds })
                    .serve(
                        self.router
                            .into_make_service_with_connect_info::<UdsConnectInfo>(),
                    )
                    .with_graceful_shutdown(self.shutdown_rx.map(drop))
                    .await
            }
        }
    }
}

impl super::Server<axum::Router, hyper::Error> for Http {
    type Handle = Handle;
    #[must_use]
    fn start(listener_config: &ConnectionInfo, router: axum::Router) -> Handle {
        let (shutdown_tx, shutdown_rx) = channel();
        let config = listener_config.clone();
        let task_handle = tokio::spawn(async move {
            let server = Http {
                config,
                router,
                shutdown_rx,
            };
            server.run().await
        });

        Handle::new(shutdown_tx, task_handle)
    }
}
