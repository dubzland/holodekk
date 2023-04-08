mod projector;
pub use projector::*;
mod uhura;
pub use uhura::*;

use std::net::SocketAddr;

use futures_util::FutureExt;
use tokio::{net::UnixListener, sync::oneshot::Receiver};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::server::TcpIncoming;

use crate::utils::{fs::cleanup, ConnectionInfo};

async fn run_server(
    listener_config: ConnectionInfo,
    server: tonic::transport::server::Router,
    shutdown: Receiver<()>,
) -> std::result::Result<(), tonic::transport::Error> {
    match listener_config {
        ConnectionInfo::Tcp { port, addr } => {
            let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();
            let listener = TcpIncoming::new(listen_address, true, None).unwrap();
            server
                .serve_with_incoming_shutdown(listener, shutdown.map(drop))
                .await
        }
        ConnectionInfo::Unix { socket } => {
            cleanup(&socket).unwrap();
            let uds = UnixListener::bind(socket).unwrap();
            let listener = UnixListenerStream::new(uds);
            server
                .serve_with_incoming_shutdown(listener, shutdown.map(drop))
                .await
        }
    }
}
