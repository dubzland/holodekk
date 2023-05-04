use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use holodekk::errors::grpc::ClientResult;

use crate::apis::grpc::uhura::ApiClient;

#[derive(Clone, Debug)]
pub struct Client {
    channel: Channel,
}

impl Client {
    fn new(channel: Channel) -> Self {
        Self { channel }
    }

    /// # Errors
    ///
    /// - Failure to generate endpoint address
    /// - Error connecting to server
    pub async fn connect_tcp(port: u16, addr: Ipv4Addr) -> ClientResult<Client> {
        let connect_address = format!("http://{addr}:{port}");
        let channel = Endpoint::from_shared(connect_address)?.connect().await?;
        Ok(Self::new(channel))
    }

    /// # Errors
    ///
    /// - Failure to generate endpoint address
    /// - Error connecting to server
    pub async fn connect_unix<P>(socket: P) -> ClientResult<Client>
    where
        P: AsRef<Path> + Into<PathBuf> + Sync + Sync,
    {
        let socket: Arc<PathBuf> = Arc::new(socket.into());
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                async move { UnixStream::connect(&*socket).await }
            }))
            .await?;
        Ok(Self::new(channel))
    }

    #[must_use]
    pub fn uhura(&self) -> ApiClient {
        ApiClient::new(self.channel.clone())
    }
}
