use std::{
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use axum::extract::connect_info;

use futures::ready;
use futures_util::FutureExt;
use hyper::server::accept::Accept;
use log::trace;
use tokio::{
    net::{unix::UCred, UnixListener, UnixStream},
    sync::oneshot::channel,
};
use tower::BoxError;

use crate::utils::{fs::cleanup, ConnectionInfo};

#[derive(Debug)]
pub struct Handle {
    shutdown_tx: RwLock<Option<tokio::sync::oneshot::Sender<()>>>,
    task_handle: RwLock<Option<tokio::task::JoinHandle<std::result::Result<(), hyper::Error>>>>,
}

impl Handle {
    pub fn new(
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
        task_handle: tokio::task::JoinHandle<std::result::Result<(), hyper::Error>>,
    ) -> Self {
        Self {
            shutdown_tx: RwLock::new(Some(shutdown_tx)),
            task_handle: RwLock::new(Some(task_handle)),
        }
    }

    pub async fn stop(&mut self) -> std::result::Result<(), hyper::Error> {
        let shutdown_tx = self.shutdown_tx.write().unwrap().take();
        if let Some(shutdown_tx) = shutdown_tx {
            shutdown_tx.send(()).unwrap();
        }
        let task_handle = self.task_handle.write().unwrap().take();
        if let Some(task_handle) = task_handle {
            task_handle.await.unwrap()?;
        }
        Ok(())
    }
}

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

#[must_use]
pub fn start(listener_config: &ConnectionInfo, server: axum::Router) -> Handle {
    let (shutdown_tx, shutdown_rx) = channel();
    let task_handle = match listener_config {
        ConnectionInfo::Tcp { port, addr } => {
            let listen_address: SocketAddr = format!("{addr}:{port}").parse().unwrap();
            tokio::spawn(async move {
                axum::Server::bind(&listen_address)
                    .serve(server.into_make_service())
                    .with_graceful_shutdown(shutdown_rx.map(drop))
                    .await
            })
        }
        ConnectionInfo::Unix { socket } => {
            cleanup(socket).unwrap();
            trace!("setting up listener at {}", socket.display());
            let uds = UnixListener::bind(socket).unwrap();
            tokio::spawn(async {
                axum::Server::builder(AcceptConnection { uds })
                    .serve(server.into_make_service_with_connect_info::<UdsConnectInfo>())
                    .with_graceful_shutdown(shutdown_rx.map(drop))
                    .await
            })
        }
    };

    Handle::new(shutdown_tx, task_handle)
}
