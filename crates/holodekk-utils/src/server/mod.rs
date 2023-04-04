//! Utilities for managing async servers in synchronous environments.
//!
//! Tokio doesn't play well with daemonizing, and signal handling is still a little rough in the
//! async realm.  The utilities in the module are meant to ease some of the pain.
//!
//! The [ServerManager] exists to run one or more homogeneous services (Tonic, Axum, etc) in a
//! background thread on their own Tokio runtime.  This frees the main thread to operate in a more
//! traditional manner with regard to signal handling and other process management.
pub mod tonic;

use std::{
    fmt,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Sender},
};

use async_trait::async_trait;
use log::warn;

#[derive(thiserror::Error, Debug)]
pub enum ListenerConfigError {
    #[error("Options for both TCP and UDS were supplied")]
    TooManyValues,
    #[error("Neither options were provided")]
    NotEnoughValues,
}

/// Configuration values necessary to construct a Listener
///
/// Rather than passing around raw values, it is easier to store them in their "final", validated
/// form in a way that can be matched on.  This makes constructing the actual listeners easier.
#[derive(Clone, Debug, PartialEq)]
pub enum ListenerConfig {
    /// TCP based socket
    Tcp { port: u16, addr: Ipv4Addr },
    /// Unix domain socket
    Uds { socket: PathBuf },
}

impl ListenerConfig {
    /// Create a listener config from a set of CLI arguments
    ///
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::{net::Ipv4Addr, path::PathBuf};
    /// use clap::Parser;
    /// use holodekk_utils::server::ListenerConfig;
    ///
    /// #[derive(Parser)]
    /// pub struct Options {
    ///     port: Option<u16>,
    ///     address: Option<Ipv4Addr>,
    ///     #[arg(conflicts_with_all = ["port", "address"])]
    ///     socket: Option<PathBuf>
    /// }
    ///
    /// fn main() {
    ///     let options = Options::parse();
    ///     let config = ListenerConfig::from_options(
    ///         options.port.as_ref(),
    ///         options.address.as_ref(),
    ///         options.socket.as_ref()
    ///     ).unwrap();
    /// }
    ///
    /// ```
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

impl fmt::Display for ListenerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ListenerConfig::Tcp { port, addr } => write!(f, "Port: {}, Address: {}", port, addr),
            ListenerConfig::Uds { socket } => write!(f, "Path: {}", socket.display()),
        }
    }
}

/// Managed server's current status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ServerStatus {
    Pending,
    Running,
    Stopped,
}

/// Represents a Server being managed by the [ServerManager].
#[async_trait]
pub trait ServerHandle: Sized + Send + 'static {
    type Result: Send + Sized + Sync + 'static;
    type Error: std::error::Error + fmt::Display + Send + Sync + 'static;

    /// Starts the server's future on the active runtime
    fn start(&mut self);
    /// Triggers the shutdown process for the server
    async fn stop(&mut self) -> std::result::Result<Self::Result, Self::Error>;
    /// Displays the server's current [ServerStatus]
    fn status(&self) -> ServerStatus;
}

/// A single server instance managed by the [ServerManager].
///
/// In essence, this will be the "thing" that is listening via port or socket.  There will almost
/// always be a 1:1 relationship between Server and a listener.
pub trait Server: Send + Sized + Sync + 'static {
    type Handle: ServerHandle;

    fn listen(&self) -> Self::Handle;
}

/// Manages the lifecycle of async-based servers (Tonic, Axum, etc).
///
/// Uses a background thread for processing to free the main thread up for general daemon
/// housekeeping and signal processing.
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
