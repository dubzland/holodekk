pub mod fs;
pub mod libsee;
pub mod logger;
pub mod pipes;
pub mod signals;
pub mod streams;

use std::fmt;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait TaskHandle {
    async fn stop(&mut self);
}

pub trait Worker: TaskHandle + Send + Sync {
    type Command;

    fn sender(&self) -> Option<tokio::sync::mpsc::Sender<Self::Command>>;
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectionInfoError {
    #[error("Options for both TCP and UDS were supplied")]
    TooManyValues,
    #[error("Neither options were provided")]
    NotEnoughValues,
    #[error("Not a Unix socket")]
    NotUnixSocket,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ConnectionInfo {
    /// TCP based socket
    Tcp { port: u16, addr: Ipv4Addr },
    /// Unix domain socket
    Unix { socket: PathBuf },
}

impl ConnectionInfo {
    pub fn from_options<P>(
        port: Option<&u16>,
        addr: Option<&Ipv4Addr>,
        socket: Option<P>,
    ) -> Result<Self, ConnectionInfoError>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        if let Some(port) = port {
            Ok(Self::tcp(port, addr))
        } else if let Some(socket) = socket {
            Ok(Self::unix(socket))
        } else {
            Err(ConnectionInfoError::NotEnoughValues)
        }
    }

    pub fn tcp(port: &u16, addr: Option<&Ipv4Addr>) -> Self {
        Self::Tcp {
            port: port.to_owned(),
            addr: addr.unwrap_or(&Ipv4Addr::new(0, 0, 0, 0)).to_owned(),
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

    pub fn to_socket(&self) -> std::result::Result<String, ConnectionInfoError> {
        match self {
            ConnectionInfo::Tcp { .. } => Err(ConnectionInfoError::NotUnixSocket),
            ConnectionInfo::Unix { socket } => Ok(socket.to_str().unwrap().to_owned()),
        }
    }
}

impl fmt::Display for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionInfo::Tcp { port, addr } => write!(f, "Port: {}, Address: {}", port, addr),
            ConnectionInfo::Unix { socket } => write!(f, "Path: {}", socket.display()),
        }
    }
}
