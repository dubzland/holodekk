use std::{
    fmt::Display,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Sender},
};

use async_trait::async_trait;

use log::warn;

pub mod tonic;

#[derive(thiserror::Error, Debug)]
pub enum ListenerConfigError {
    #[error("Options for both TCP and UDS were supplied")]
    TooManyValues,
    #[error("Neither options were provided")]
    NotEnoughValues,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListenerConfig {
    Tcp { port: u16, addr: Ipv4Addr },
    Uds { socket: PathBuf },
}

impl ListenerConfig {
    pub fn from_options<P: AsRef<Path>>(
        port: Option<&u16>,
        addr: Option<&Ipv4Addr>,
        socket: Option<P>,
    ) -> Result<Self, ListenerConfigError> {
        if let Some(port) = port {
            Ok(Self::tcp(port, addr))
        } else if let Some(socket) = socket {
            Ok(Self::uds(socket))
        } else {
            Err(ListenerConfigError::NotEnoughValues)
        }
    }

    pub fn tcp(port: &u16, addr: Option<&Ipv4Addr>) -> Self {
        Self::Tcp {
            port: port.to_owned(),
            addr: addr.unwrap_or(&Ipv4Addr::new(0, 0, 0, 0)).to_owned(),
        }
    }

    pub fn uds<P: AsRef<Path>>(socket: P) -> Self {
        Self::Uds {
            socket: socket.as_ref().to_owned(),
        }
    }
}

#[async_trait]
pub trait ServerHandle: Sized + Send + 'static {
    type Result: Send + Sync + 'static;
    type Error: std::error::Error + Display + Send + Sync + 'static;

    fn start(&mut self);
    async fn stop(&mut self) -> std::result::Result<Self::Result, Self::Error>;
}

pub trait Server: Send + Sized + Sync + 'static {
    type Handle: ServerHandle;

    fn listen(&self) -> Self::Handle;
}

pub trait ServerManager: Send + Sized + 'static {
    type Server: Server;

    fn new(thread_handle: std::thread::JoinHandle<()>, shutdown_tx: Sender<()>) -> Self;
    fn shutdown_tx(&self) -> Option<Sender<()>>;
    fn thread_handle(&self) -> Option<std::thread::JoinHandle<()>>;

    fn start(servers: Vec<Self::Server>) -> Self {
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        let (status_tx, status_rx) = channel::<()>();

        let thread_handle = std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _guard = runtime.enter();
            let runtime_handle = tokio::runtime::Handle::current();

            let mut handles: Vec<<<Self as ServerManager>::Server as Server>::Handle> =
                servers.iter().map(|s| s.listen()).collect();

            let _guard = runtime_handle.enter();

            for handle in handles.iter_mut() {
                handle.start();
            }

            // notify
            status_tx.send(()).unwrap();

            // wait for shutdown signal
            shutdown_rx.recv().unwrap();

            runtime_handle.block_on(async move {
                for handle in handles.iter_mut() {
                    if let Err(err) = handle.stop().await {
                        warn!("Server failed: {}", err);
                    }
                }
            });
        });

        // wait until servers are ready
        status_rx.recv().unwrap();

        Self::new(thread_handle, shutdown_tx)
    }

    fn stop(&self) {
        if let Some(shutdown_tx) = self.shutdown_tx() {
            shutdown_tx.send(()).unwrap();
        }

        if let Some(thread_handle) = self.thread_handle() {
            thread_handle.join().unwrap();
        }
    }
}

// pub struct ServerManagerOld {
//     thread_handle: RwLock<Option<std::thread::JoinHandle<()>>>,
//     cmd_tx: Sender<()>,
// }

// impl ServerManagerOld {
//     fn new(thread_handle: std::thread::JoinHandle<()>, cmd_tx: Sender<()>) -> Self {
//         Self {
//             thread_handle: RwLock::new(Some(thread_handle)),
//             cmd_tx,
//         }
//     }

//     pub fn start<S, H, T, E>(
//         servers: Vec<S>,
//     ) -> std::result::Result<ServerManagerOld, std::io::Error>
//     where
//         S: Server<Handle = H>,
//         H: ServerHandle<Result = T, Error = E>,
//     {
//         let (shutdown_tx, shutdown_rx) = channel();
//         let (status_tx, status_rx) = channel();

//         let thread_handle = std::thread::spawn(move || {
//             let runtime = tokio::runtime::Runtime::new().unwrap();
//             let _guard = runtime.enter();
//             let runtime_handle = tokio::runtime::Handle::current();

//             let mut handles: Vec<H> = servers.iter().map(|s| s.listen()).collect();

//             let _guard = runtime_handle.enter();

//             for handle in handles.iter_mut() {
//                 handle.start();
//             }

//             // notify
//             status_tx.send(()).unwrap();

//             // wait for shutdown signal
//             let _ = shutdown_rx.recv().unwrap();

//             runtime_handle.block_on(async move {
//                 for handle in handles.iter_mut() {
//                     if let Err(err) = handle.stop().await {
//                         warn!("Server failed: {}", err);
//                     }
//                 }
//             });
//         });

//         // wait until servers are ready
//         status_rx.recv().unwrap();

//         Ok(ServerManagerOld::new(thread_handle, shutdown_tx))
//     }

//     pub fn stop(&self) {
//         self.cmd_tx.send(()).unwrap();
//         if self.thread_handle.read().unwrap().is_some() {
//             let thread_handle = self.thread_handle.write().unwrap().take().unwrap();
//             thread_handle.join().unwrap();
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn listener_config_from_options_works_for_port() -> Result<(), ListenerConfigError> {
        match ListenerConfig::from_options(Some(&1), None, None::<PathBuf>)? {
            ListenerConfig::Tcp { port, addr } => {
                assert_eq!(port, 1);
                assert_eq!(addr, Ipv4Addr::new(0, 0, 0, 0));
            }
            _ => panic!("Wrong type returned by tcp_listener_kind()"),
        }
        Ok(())
    }

    #[test]
    fn listener_config_from_options_accepts_an_address() -> Result<(), ListenerConfigError> {
        let addr = Ipv4Addr::new(127, 0, 0, 1);
        match ListenerConfig::from_options(Some(&1), Some(&addr), None::<PathBuf>)? {
            ListenerConfig::Tcp { port, addr } => {
                assert_eq!(port, 1);
                assert_eq!(addr, Ipv4Addr::new(127, 0, 0, 1));
            }
            _ => panic!("Wrong type returned by tcp_listener_kind()"),
        }
        Ok(())
    }

    #[test]
    fn listener_config_from_options_accepts_a_socket() -> Result<(), ListenerConfigError> {
        let socket = PathBuf::from("/tmp/temp.sock");
        match ListenerConfig::from_options(None, None, Some(&socket))? {
            ListenerConfig::Uds { socket } => {
                assert_eq!(socket, Path::new("/tmp/temp.sock"));
            }
            _ => panic!("Wrong type returned by tcp_listener_kind()"),
        }
        Ok(())
    }
}
