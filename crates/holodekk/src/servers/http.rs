use std::{
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
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

pub struct HttpServerHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    task_handle: tokio::task::JoinHandle<std::result::Result<(), hyper::Error>>,
}

impl HttpServerHandle {
    pub fn new(
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
        task_handle: tokio::task::JoinHandle<std::result::Result<(), hyper::Error>>,
    ) -> Self {
        Self {
            shutdown_tx,
            task_handle,
        }
    }

    pub async fn stop(self) -> std::result::Result<(), hyper::Error> {
        self.shutdown_tx.send(()).unwrap();
        self.task_handle.await.unwrap()
    }
}

struct ServerAccept {
    uds: UnixListener,
}

impl Accept for ServerAccept {
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

pub fn start_http_server(
    listener_config: &ConnectionInfo,
    server: axum::Router,
) -> HttpServerHandle {
    let (shutdown_tx, shutdown_rx) = channel();
    let task_handle = match listener_config {
        ConnectionInfo::Tcp { port, addr } => {
            let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();
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
                axum::Server::builder(ServerAccept { uds })
                    .serve(server.into_make_service_with_connect_info::<UdsConnectInfo>())
                    .with_graceful_shutdown(shutdown_rx.map(drop))
                    .await
            })
        }
    };

    HttpServerHandle::new(shutdown_tx, task_handle)
}
