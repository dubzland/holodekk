use std::{
    cell::RefCell,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use futures_util::FutureExt;

use tokio::{
    net::UnixListener,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};

use tokio_stream::wrappers::UnixListenerStream;

use tonic::transport::server::TcpIncoming;

#[derive(Debug)]
pub enum ApiServerCommand {
    Stop {
        completion: Option<oneshot::Sender<()>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApiListenerKind {
    Tcp { port: u16, addr: Ipv4Addr },
    Uds { socket: PathBuf },
}

pub trait ApiService {
    fn to_router(&self) -> tonic::transport::server::Router;
}

#[derive(Debug)]
pub struct ApiServer<S>
where
    S: ApiService + Send + Sync + 'static,
{
    service: S,
    listener: ApiListenerKind,
    cmd_tx: RefCell<Option<UnboundedSender<ApiServerCommand>>>,
    handle: RefCell<Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>>,
}

impl<S> ApiServer<S>
where
    S: ApiService + Send + Sync + 'static,
{
    pub fn new(service: S, listener: ApiListenerKind) -> Self {
        Self {
            service,
            listener,
            cmd_tx: RefCell::new(None),
            handle: RefCell::new(None),
        }
    }

    pub fn listen_tcp(service: S, port: &u16, addr: Option<&Ipv4Addr>) -> Self
    where
        S: ApiService,
    {
        let default = Ipv4Addr::new(0, 0, 0, 0);
        let addr = addr.unwrap_or(&default);
        let listener = ApiListenerKind::Tcp {
            port: port.to_owned(),
            addr: addr.to_owned(),
        };
        Self::new(service, listener)
    }

    pub fn listen_uds<P: AsRef<Path>>(service: S, socket: P) -> Self {
        // clean up if necessary
        let socket = socket.as_ref();
        if socket.exists() {
            std::fs::remove_file(socket).expect("Failed to remove existing listening socket");
        }
        let listener = ApiListenerKind::Uds {
            socket: socket.to_owned(),
        };
        Self::new(service, listener)
    }

    pub fn start(&self) {
        let (cmd_tx, cmd_rx) = unbounded_channel();
        self.cmd_tx.borrow_mut().replace(cmd_tx);
        let shutdown = shutdown_signal(cmd_rx);

        let router = self.service.to_router();

        let handle = match self.listener.clone() {
            ApiListenerKind::Tcp { port, addr } => {
                let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();
                let listener = TcpIncoming::new(listen_address, true, None).unwrap();
                tokio::spawn(router.serve_with_incoming_shutdown(listener, shutdown.map(drop)))
            }
            ApiListenerKind::Uds { socket } => {
                let uds = UnixListener::bind(socket).unwrap();
                let listener = UnixListenerStream::new(uds);
                tokio::spawn(router.serve_with_incoming_shutdown(listener, shutdown.map(drop)))
            }
        };

        self.handle.borrow_mut().replace(handle);
    }

    pub async fn stop(&self) -> std::result::Result<(), tokio::sync::oneshot::error::RecvError> {
        let (status_tx, status_rx) = oneshot::channel();
        let cmd_tx = self.cmd_tx.borrow_mut().take().unwrap();
        cmd_tx
            .send(ApiServerCommand::Stop {
                completion: Some(status_tx),
            })
            .expect("Failed to send shutdown request to server");
        status_rx.await?;
        let handle = self.handle.replace(None).unwrap();
        let res = handle.await;
        match res {
            Ok(res) => {
                res.unwrap();
            }
            Err(err) => {
                tracing::warn!("Error waiting for ApiServer to shutdown: {}", err);
            }
        }
        Ok(())
    }
}

async fn shutdown_signal(cmd_rx: UnboundedReceiver<ApiServerCommand>) {
    let mut cmd_rx = cmd_rx;
    let cmd = cmd_rx.recv().await.unwrap();
    match cmd {
        ApiServerCommand::Stop { completion } => {
            if let Some(tx) = completion {
                let _ = tx.send(());
            }
        }
    };
}
