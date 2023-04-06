pub mod fs;
pub mod libsee;
pub mod logger;
pub mod pipes;
pub mod signals;
pub mod streams;

use std::fmt;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

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
