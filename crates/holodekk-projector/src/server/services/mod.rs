use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use futures_util::FutureExt;

use log::{debug, warn};

use tokio::net::UnixListener;
use tokio::sync::oneshot::{channel, Sender};
use tokio::task::JoinHandle;

use tokio_stream::wrappers::UnixListenerStream;

use tonic::transport::server::TcpIncoming;

use crate::{Error, Result};

pub mod admin;

pub type RouterFactory = fn() -> tonic::transport::server::Router;

#[derive(Debug)]
pub enum ServiceCommand {
    Stop {
        completion: Option<tokio::sync::oneshot::Sender<()>>,
    },
}

pub type ServiceHandle = JoinHandle<std::result::Result<(), tonic::transport::Error>>;

#[derive(Clone, Debug)]
pub struct Service {
    router: RouterFactory,
    port: Option<u16>,
    address: Option<Ipv4Addr>,
    socket: Option<PathBuf>,
    cmd_tx: Arc<RwLock<Option<Sender<ServiceCommand>>>>,
}

impl Service {
    fn new(router: RouterFactory) -> Self {
        Self {
            router,
            port: None,
            address: None,
            socket: None,
            cmd_tx: Arc::new(RwLock::new(None)),
        }
    }
    pub fn admin() -> Self {
        Self::new(admin::router)
    }
    pub fn projector() -> Self {
        Self::new(admin::router)
    }

    pub fn listen_tcp(&mut self, port: &u16, address: Option<&Ipv4Addr>) -> &mut Self {
        let default = Ipv4Addr::new(0, 0, 0, 0);
        let address = address.unwrap_or(&default);

        self.address = Some(address.clone().to_owned());

        self.port = Some(port.to_owned());
        self
    }

    pub fn listen_uds(&mut self, path: &PathBuf) -> &mut Self {
        self.socket = Some(path.to_owned());
        // clean up if necessary
        if self.socket.as_ref().unwrap().exists() {
            std::fs::remove_file(self.socket.as_ref().unwrap())
                .expect("Failed to remove existing listening socket");
        }
        self
    }

    pub fn start(&self) -> Result<ServiceHandle> {
        let (cmd_tx, cmd_rx) = channel();

        let signal = async {
            let res = cmd_rx.await;
            match res {
                Ok(cmd) => match cmd {
                    ServiceCommand::Stop { completion } => {
                        if let Some(tx) = completion {
                            let _ = tx.send(());
                        }
                    }
                },
                Err(err) => {
                    warn!("Error received checking signal message: {}", err);
                }
            }
        };

        let router_factory = self.router;
        let router = router_factory();

        let task_handle = if self.port.is_some() {
            let default = Ipv4Addr::new(0, 0, 0, 0);
            let address = self.address.unwrap_or(default);
            let listen_address: SocketAddr = format!("{}:{}", address, self.port.unwrap())
                .parse()
                .unwrap();

            let listener = TcpIncoming::new(listen_address, true, None).unwrap();

            tokio::spawn(async move {
                debug!("Inside spawn of server.");
                let res = router
                    .serve_with_incoming_shutdown(listener, signal.map(drop))
                    .await;
                debug!("server shutdown");
                res
            })
        } else {
            let uds = UnixListener::bind(self.socket.as_ref().unwrap()).unwrap();
            let listener = UnixListenerStream::new(uds);

            tokio::spawn(async move {
                debug!("Inside spawn of server.");
                router
                    .serve_with_incoming_shutdown(listener, signal.map(drop))
                    .await
            })
        };
        // } else {
        //     let listener = self.uds_listener.write().unwrap().take().unwrap();
        //     tokio::spawn(async move {
        //         debug!("Inside spawn of server.");
        //         router
        //             .serve_with_incoming_shutdown(listener, signal.map(drop))
        //             .await
        //     })
        // };

        self.cmd_tx.write().unwrap().replace(cmd_tx);

        Ok(task_handle)
    }

    pub async fn stop(&self) -> Result<()> {
        let (status_tx, status_rx) = channel();
        let cmd_tx = self.cmd_tx.write().unwrap().take().unwrap();
        cmd_tx
            .send(ServiceCommand::Stop {
                completion: Some(status_tx),
            })
            .expect("Failed to send shutdown request to server");
        status_rx.await.map_err(|_| Error::Shutdown)?;
        Ok(())
    }
}
