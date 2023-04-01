use std::{fmt, net::Ipv4Addr, path::Path, result::Result, sync::Arc};

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use holodekk_utils::errors::error_chain_fmt;

use super::proto::{
    core::CoreClient,
    entities::{Empty, ProjectorStatus},
    subroutines::SubroutinesClient,
};

#[derive(thiserror::Error)]
pub enum UhuraError {
    #[error("Failed to connect to server")]
    Transport(#[from] tonic::transport::Error),
    #[error("Failed to execute RPC request")]
    Status(#[from] tonic::Status),
}

impl std::fmt::Debug for UhuraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type UhuraResult<T> = Result<T, UhuraError>;

#[derive(Clone, Debug)]
pub struct CoreClientWrapper {
    inner: CoreClient<Channel>,
}

impl CoreClientWrapper {
    pub fn new(client: CoreClient<Channel>) -> Self {
        Self { inner: client }
    }

    pub async fn status(&self) -> UhuraResult<ProjectorStatus> {
        let req = tonic::Request::new(Empty {});
        let mut client = self.inner.clone();
        let response = client.status(req).await?;
        Ok(response.into_inner())
    }
}

#[derive(Clone, Debug)]
pub struct UhuraClient {
    channel: Channel,
}

impl UhuraClient {
    fn new(channel: Channel) -> Self {
        Self { channel }
    }

    pub async fn connect_tcp(port: u16, addr: Ipv4Addr) -> UhuraResult<UhuraClient> {
        let connect_address = format!("http://{}:{}", addr, port);
        let channel = Endpoint::from_shared(connect_address)?.connect().await?;
        Ok(Self::new(channel))
    }

    pub async fn connect_uds<P: AsRef<Path>>(socket: P) -> UhuraResult<UhuraClient> {
        let socket = Arc::new(socket.as_ref().to_owned());
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                async move { UnixStream::connect(&*socket).await }
            }))
            .await?;
        Ok(Self::new(channel))
    }

    pub fn core(&self) -> CoreClientWrapper {
        let client = CoreClient::new(self.channel.clone());
        CoreClientWrapper::new(client)
    }

    pub fn subroutines(&self) -> SubroutinesClient<Channel> {
        SubroutinesClient::new(self.channel.clone())
    }
}
