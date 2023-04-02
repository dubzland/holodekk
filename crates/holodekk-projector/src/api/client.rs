use std::fmt;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use holodekk_utils::errors::error_chain_fmt;

use super::proto::applications::RpcApplicationsClient;
use super::proto::entities::RpcEmpty;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Failed to connect to server")]
    Transport(#[from] tonic::transport::Error),
    #[error("Failed to execute RPC request")]
    Status(#[from] tonic::Status),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct ApplicationsClient {
    inner: RpcApplicationsClient<Channel>,
}

impl ApplicationsClient {
    pub fn new(client: RpcApplicationsClient<Channel>) -> Self {
        Self { inner: client }
    }

    pub async fn list(&self) -> Result<String> {
        let mut client = self.inner.clone();
        let request = tonic::Request::new(RpcEmpty {});
        let response = client.list(request).await?;
        Ok(response.into_inner().message)
    }
}

#[derive(Clone, Debug)]
pub struct ProjectorClient {
    channel: Channel,
}

impl ProjectorClient {
    fn new(channel: Channel) -> Self {
        Self { channel }
    }

    pub async fn connect_tcp(port: u16, addr: Ipv4Addr) -> Result<ProjectorClient> {
        let connect_address = format!("http://{}:{}", addr, port);
        let channel = Endpoint::from_shared(connect_address)?.connect().await?;
        Ok(Self::new(channel))
    }

    pub async fn connect_uds<P: AsRef<Path>>(socket: P) -> Result<ProjectorClient> {
        let socket = Arc::new(socket.as_ref().to_owned());
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                async move { UnixStream::connect(&*socket).await }
            }))
            .await?;
        Ok(Self::new(channel))
    }

    pub fn applications(&self) -> ApplicationsClient {
        let client = RpcApplicationsClient::new(self.channel.clone());
        ApplicationsClient::new(client)
    }
}
