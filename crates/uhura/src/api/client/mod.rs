use std::{net::Ipv4Addr, path::Path, sync::Arc};

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use holodekk::apis::grpc::subroutines::SubroutinesClient;
use holodekk::errors::grpc::GrpcClientResult;

mod core;
pub use self::core::*;

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

    pub async fn connect_uds<P: AsRef<Path>>(socket: P) -> GrpcClientResult<UhuraClient> {
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
        CoreClient::new(self.channel.clone())
    }

    pub fn subroutines(&self) -> SubroutinesClient {
        SubroutinesClient::new(self.channel.clone())
    }
}
