use std::{fmt, net::Ipv4Addr, path::Path, sync::Arc};

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use holodekk_utils::errors::error_chain_fmt;

use super::proto::{core::RpcCoreClient, subroutines::RpcSubroutinesClient};

mod core;
pub use self::core::*;
mod subroutines;
pub use self::subroutines::*;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Failed to connect to server")]
    Transport(#[from] tonic::transport::Error),
    #[error("Failed to execute RPC request")]
    Status(#[from] tonic::Status),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct UhuraClient {
    channel: Channel,
}

impl UhuraClient {
    fn new(channel: Channel) -> Self {
        Self { channel }
    }

    pub async fn connect_tcp(port: u16, addr: Ipv4Addr) -> Result<UhuraClient> {
        let connect_address = format!("http://{}:{}", addr, port);
        let channel = Endpoint::from_shared(connect_address)?.connect().await?;
        Ok(Self::new(channel))
    }

    pub async fn connect_uds<P: AsRef<Path>>(socket: P) -> Result<UhuraClient> {
        let socket = Arc::new(socket.as_ref().to_owned());
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                async move { UnixStream::connect(&*socket).await }
            }))
            .await?;
        Ok(Self::new(channel))
    }

    pub fn core(&self) -> CoreClient {
        let client = RpcCoreClient::new(self.channel.clone());
        CoreClient::new(client)
    }

    pub fn subroutines(&self) -> SubroutinesClient {
        let client = RpcSubroutinesClient::new(self.channel.clone());
        SubroutinesClient::new(client)
    }
}
