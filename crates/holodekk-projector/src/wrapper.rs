use std::cell::RefCell;
use std::net::{SocketAddr, TcpListener};
use std::thread::{self, JoinHandle};

use futures_util::FutureExt;

use log::{debug, warn};

use tokio::runtime::Handle;
use tokio::sync::oneshot::{channel, Sender};

use tonic::transport::server::Router;

use crate::{Error, Result};

#[derive(Debug)]
pub enum ServerCommand {
    Stop {
        completion: Option<tokio::sync::oneshot::Sender<()>>,
    },
}

pub struct ServerHandle {
    thread_handle: RefCell<Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>>,
    cmd_tx: RefCell<Option<Sender<ServerCommand>>>,
    address: SocketAddr,
}

impl ServerHandle {
    pub fn new(
        thread_handle: JoinHandle<std::result::Result<(), tonic::transport::Error>>,
        cmd_tx: Sender<ServerCommand>,
        address: SocketAddr,
    ) -> Self {
        Self {
            thread_handle: RefCell::new(Some(thread_handle)),
            cmd_tx: RefCell::new(Some(cmd_tx)),
            address,
        }
    }
    pub fn port(&self) -> u16 {
        self.address.port()
    }

    fn stop(&self, channel: Sender<()>) -> Result<()> {
        let cmd_tx = self.cmd_tx.take().unwrap();
        let thread_handle = self.thread_handle.take().unwrap();
        cmd_tx
            .send(ServerCommand::Stop {
                completion: Some(channel),
            })
            .expect("Failed to send shutdown request to server");

        thread_handle
            .join()
            .expect("Failed to wait for server thread completion")?;
        Ok(())
    }
}

pub struct ServerManager {}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerManager {
    pub fn new() -> Self {
        Self {}
    }

    fn start(&self, router: Router, address: SocketAddr) -> crate::Result<ServerHandle> {
        let (cmd_tx, cmd_rx) = channel();

        let signal = async {
            let res = cmd_rx.await;
            match res {
                Ok(cmd) => match cmd {
                    ServerCommand::Stop { completion } => {
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

        let runtime_handle = Handle::current();
        let thread_handle = thread::spawn(move || {
            let res = runtime_handle.block_on(async move {
                debug!("Inside spawn of server.");
                router
                    .serve_with_shutdown(address, signal.map(drop))
                    .await?;
                Ok(())
            });
            debug!("block returned");
            res
        });
        debug!("launched");

        Ok(ServerHandle::new(thread_handle, cmd_tx, address))
    }

    pub fn start_tcp(
        &self,
        router: Router,
        port: Option<u16>,
        addr: Option<&str>,
    ) -> crate::Result<ServerHandle> {
        let addr = addr.unwrap_or("[::1]");
        let port = port
            .or_else(|| {
                let listener = TcpListener::bind(format!("{addr}:0")).unwrap();
                Some(listener.local_addr().unwrap().port())
            })
            .unwrap();
        let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();

        self.start(router, listen_address)
    }

    pub async fn stop(&self, handle: &ServerHandle) -> Result<()> {
        let (status_tx, status_rx) = channel();

        handle.stop(status_tx)?;

        status_rx.await.map_err(|_| Error::Shutdown)?;
        Ok(())
    }
}
