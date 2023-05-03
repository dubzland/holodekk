pub mod fs;
pub mod grpc;
pub mod http;
pub mod libsee;
pub mod logger;
pub mod pipes;
pub mod process;
pub mod signals;
pub mod streams;

use std::fmt;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum ConnectionInfoError {
    #[error("Options for both TCP and UDS were supplied")]
    TooManyValues,
    #[error("Neither options were provided")]
    NotEnoughValues,
    #[error("Not a Unix socket")]
    NotUnixSocket,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ConnectionInfo {
    /// TCP based socket
    Tcp { port: u16, addr: Ipv4Addr },
    /// Unix domain socket
    Unix { socket: PathBuf },
}

impl ConnectionInfo {
    #[must_use]
    pub fn tcp(port: &u16, addr: Option<&Ipv4Addr>) -> Self {
        Self::Tcp {
            port: port.to_owned(),
            addr: *addr.unwrap_or(&Ipv4Addr::new(0, 0, 0, 0)),
        }
    }

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
