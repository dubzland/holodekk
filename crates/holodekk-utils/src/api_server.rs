use std::{
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use futures_util::FutureExt;

use log::error;

use tokio::{
    net::UnixListener,
    sync::oneshot::{channel, Sender},
    task::JoinHandle,
};

use tokio_stream::wrappers::UnixListenerStream;

use tonic::transport::server::TcpIncoming;

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
    cmd_tx: Arc<RwLock<Option<Sender<()>>>>,
}

impl<S> ApiServer<S>
where
    S: ApiService + Send + Sync + 'static,
{
    pub fn new(service: S, listener: ApiListenerKind) -> Self {
        Self {
            service,
            listener,
            cmd_tx: Arc::new(RwLock::new(None)),
        }
    }

    pub fn listener(&self) -> ApiListenerKind {
        self.listener.clone()
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

    pub fn start(&self) -> JoinHandle<std::result::Result<(), tonic::transport::Error>> {
        let (cmd_tx, cmd_rx) = channel();
        self.cmd_tx.write().unwrap().replace(cmd_tx);

        let router = self.service.to_router();

        let handle = tokio::runtime::Handle::current();
        match self.listener.clone() {
            ApiListenerKind::Tcp { port, addr } => {
                let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();
                let listener = TcpIncoming::new(listen_address, true, None).unwrap();
                handle.spawn(router.serve_with_incoming_shutdown(listener, cmd_rx.map(drop)))
            }
            ApiListenerKind::Uds { socket } => {
                let uds = UnixListener::bind(socket).unwrap();
                let listener = UnixListenerStream::new(uds);
                handle.spawn(router.serve_with_incoming_shutdown(listener, cmd_rx.map(drop)))
            }
        }
    }

    pub fn stop(&self) {
        if let Some(cmd_tx) = self.cmd_tx.write().unwrap().take() {
            if cmd_tx.send(()).is_err() {
                error!("Failed to send shutdown request to server");
            }
        }
    }
}

impl<S> Drop for ApiServer<S>
where
    S: ApiService + Send + Sync + 'static,
{
    fn drop(&mut self) {
        // call stop just to be sure
        self.stop();
    }
}
