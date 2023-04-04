use std::{
    cell::RefCell,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::mpsc::Sender,
};

use async_trait::async_trait;

pub use futures_core::future::BoxFuture;
use futures_util::FutureExt;

use log::{debug, error, warn};

use tokio::{
    net::UnixListener,
    sync::oneshot::{channel as tokio_channel, Receiver as TokioReceiver, Sender as TokioSender},
    task::JoinHandle as TokioJoinHandle,
};
use tonic::transport::server::TcpIncoming;

use tokio_stream::wrappers::UnixListenerStream;

use super::{ListenerConfig, Server, ServerHandle, ServerManager};
use crate::fs::cleanup;

pub type TonicResult = std::result::Result<(), tonic::transport::Error>;

pub type TonicJoinHandle = TokioJoinHandle<TonicResult>;

pub struct TonicServerHandle {
    cmd_tx: Option<TokioSender<()>>,
    fut: Option<BoxFuture<'static, TonicResult>>,
    handle: Option<TonicJoinHandle>,
}

impl TonicServerHandle {
    pub fn new(cmd_tx: TokioSender<()>, fut: BoxFuture<'static, TonicResult>) -> Self {
        Self {
            cmd_tx: Some(cmd_tx),
            fut: Some(fut),
            handle: None,
        }
    }
}

#[async_trait]
impl ServerHandle for TonicServerHandle {
    type Result = ();
    type Error = tonic::transport::Error;

    fn start(&mut self) {
        let runtime_handle = tokio::runtime::Handle::current();
        let fut = self.fut.take().unwrap();
        let handle = runtime_handle.spawn(async {
            debug!("Awaiting fut");
            let res = fut.await;
            debug!("Awaited");
            res
        });
        self.handle.replace(handle);
    }

    async fn stop(&mut self) -> std::result::Result<Self::Result, Self::Error> {
        if self.cmd_tx.take().unwrap().send(()).is_err() {
            error!("Failed to send shutdown request to server");
        }
        let handle = self.handle.take().unwrap();
        match handle.await {
            Ok(res) => res,
            Err(err) => {
                warn!("Failed to await service handle: {}", err);
                Ok(())
            }
        }
    }
}

pub trait TonicService
where
    Self: Send + Sync + 'static,
{
    fn to_router(&self) -> tonic::transport::server::Router;
    fn listen(
        &self,
        listener_config: &ListenerConfig,
        shutdown: TokioReceiver<()>,
    ) -> BoxFuture<'static, TonicResult> {
        match listener_config {
            ListenerConfig::Tcp { port, addr } => {
                let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();
                let listener = TcpIncoming::new(listen_address, true, None).unwrap();
                Box::pin(
                    self.to_router()
                        .serve_with_incoming_shutdown(listener, shutdown.map(drop)),
                )
            }
            ListenerConfig::Uds { socket } => {
                cleanup(socket).unwrap();
                let uds = UnixListener::bind(socket).unwrap();
                let listener = UnixListenerStream::new(uds);
                Box::pin(
                    self.to_router()
                        .serve_with_incoming_shutdown(listener, shutdown.map(drop)),
                )
            }
        }
    }
}

pub struct TonicServerBuilder<T>
where
    T: TonicService,
{
    service: T,
}

impl<T> TonicServerBuilder<T>
where
    T: TonicService,
{
    pub fn new(service: T) -> Self {
        Self { service }
    }

    pub fn listen_tcp(self, port: &u16, addr: Option<&Ipv4Addr>) -> TonicServer {
        self.build(ListenerConfig::tcp(port, addr))
    }

    pub fn listen_uds<P: AsRef<Path>>(self, socket: P) -> TonicServer {
        self.build(ListenerConfig::uds(socket))
    }

    pub fn listen<P: AsRef<Path>>(
        self,
        port: Option<&u16>,
        addr: Option<&Ipv4Addr>,
        socket: Option<P>,
    ) -> TonicServer {
        self.build(ListenerConfig::from_options(port, addr, socket).unwrap())
    }

    fn build(self, listener_config: ListenerConfig) -> TonicServer {
        TonicServer::new(listener_config, self.service)
    }
}

pub struct TonicServer {
    listener_config: ListenerConfig,
    service: Box<dyn TonicService>,
}

impl TonicServer {
    fn new(listener_config: ListenerConfig, service: impl TonicService) -> Self {
        Self {
            listener_config,
            service: Box::new(service),
        }
    }
}

impl Server for TonicServer {
    type Handle = TonicServerHandle;

    fn listen(&self) -> TonicServerHandle {
        let (cmd_tx, cmd_rx) = tokio_channel::<()>();
        let fut = self.service.listen(&self.listener_config, cmd_rx);
        TonicServerHandle::new(cmd_tx, fut)
    }
}

pub struct TonicServerManager {
    thread_handle: RefCell<Option<std::thread::JoinHandle<()>>>,
    shutdown_tx: RefCell<Option<Sender<()>>>,
}

impl ServerManager for TonicServerManager {
    type Server = TonicServer;

    fn new(thread_handle: std::thread::JoinHandle<()>, shutdown_tx: Sender<()>) -> Self {
        Self {
            thread_handle: RefCell::new(Some(thread_handle)),
            shutdown_tx: RefCell::new(Some(shutdown_tx)),
        }
    }

    fn shutdown_tx(&self) -> Option<Sender<()>> {
        self.shutdown_tx.borrow_mut().take()
    }

    fn thread_handle(&self) -> Option<std::thread::JoinHandle<()>> {
        self.thread_handle.borrow_mut().take()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::sync::oneshot::{
        channel as tokio_channel, Receiver as TokioReceiver, Sender as TokioSender,
    };

    async fn test_router(cmd_rx: TokioReceiver<()>, data_tx: TokioSender<i32>) -> TonicResult {
        cmd_rx.await.unwrap();
        data_tx.send(42).unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn tonic_server_handle_runs_future() {
        let (cmd_tx, cmd_rx) = tokio_channel::<()>();
        let (data_tx, data_rx) = tokio_channel::<i32>();

        let fut = Box::pin(test_router(cmd_rx, data_tx));

        let mut handle = TonicServerHandle::new(cmd_tx, fut);
        handle.start();
        handle.stop().await.unwrap();
        let answer = data_rx.await.unwrap();
        assert_eq!(answer, 42);
    }
}
