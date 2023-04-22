use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use holodekk::errors::grpc::GrpcClientResult;

use crate::apis::grpc::uhura::UhuraApiClient;

#[derive(Clone, Debug)]
pub struct UhuraClient {
    channel: Channel,
}

impl UhuraClient {
    fn new(channel: Channel) -> Self {
        Self { channel }
    }

    pub async fn connect_tcp(port: u16, addr: Ipv4Addr) -> GrpcClientResult<UhuraClient> {
        let connect_address = format!("http://{}:{}", addr, port);
        let channel = Endpoint::from_shared(connect_address)?.connect().await?;
        Ok(Self::new(channel))
    }

    pub async fn connect_unix<P>(socket: P) -> GrpcClientResult<UhuraClient>
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

    pub fn uhura(&self) -> UhuraApiClient {
        UhuraApiClient::new(self.channel.clone())
    }
}
