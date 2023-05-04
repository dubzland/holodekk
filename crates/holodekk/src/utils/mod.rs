//! System-wide utilities

use std::fmt;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Simple structure to represent the details for a given connection.
///
/// Used as both an input (to specify configuration) and an output (to provide connection
/// details).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ConnectionInfo {
    /// TCP based socket
    Tcp {
        /// Numeric tcp port number
        port: u16,
        /// IPV4 Address (can be 0.0.0.0 to represent all interfaces)
        addr: Ipv4Addr,
    },
    /// Unix domain socket
    Unix {
        /// Filesystem path to the Unix socket file.
        socket: PathBuf,
    },
}

impl ConnectionInfo {
    /// Create a [`ConnectionInfo`] based on the given port.
    ///
    /// Optionally accepts an IPV4 address.  Will default to `0.0.0.0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use holodekk::utils::ConnectionInfo;
    ///
    /// let tcp = ConnectionInfo::tcp(&1234, None);
    /// ```
    #[must_use]
    pub fn tcp(port: &u16, addr: Option<&Ipv4Addr>) -> Self {
        Self::Tcp {
            port: port.to_owned(),
            addr: *addr.unwrap_or(&Ipv4Addr::new(0, 0, 0, 0)),
        }
    }

    /// Create a [`ConnectionInfo`] instance based on the given socket path.
    ///
    /// The path does not need to exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use holodekk::utils::ConnectionInfo;
    ///
    /// let udp = ConnectionInfo::unix("/var/lib/holodekk.sock");
    /// ```
    #[must_use]
    pub fn unix<P>(socket: P) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        Self::Unix {
            socket: socket.into(),
        }
    }
}

impl fmt::Display for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionInfo::Tcp { port, addr } => write!(f, "Port: {port}, Address: {addr}"),
            ConnectionInfo::Unix { socket } => write!(f, "Path: {}", socket.display()),
        }
    }
}

pub mod fs;
pub mod libsee;
pub mod logger;
pub mod pipes;
pub mod process;
pub mod server;
pub use server::Server;
pub mod signals;
pub mod streams;
