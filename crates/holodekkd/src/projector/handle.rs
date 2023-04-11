use std::fmt;
use uuid::Uuid;

use holodekk::errors::grpc::GrpcClientResult;
use holodekk::utils::ConnectionInfo;
use uhura::client::UhuraClient;

#[derive(Clone, Debug)]
pub struct ProjectorHandle {
    pub id: Uuid,
    pub fleet: String,
    pub namespace: String,
    pub address: ConnectionInfo,
}

impl fmt::Display for ProjectorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl ProjectorHandle {
    pub fn new(id: &Uuid, fleet: &str, namespace: &str, address: &ConnectionInfo) -> Self {
        Self {
            id: id.to_owned(),
            fleet: fleet.to_owned(),
            namespace: namespace.to_owned(),
            address: address.to_owned(),
        }
    }

    pub async fn client(&self) -> GrpcClientResult<UhuraClient> {
        let client = match &self.address {
            ConnectionInfo::Tcp { port, addr } => UhuraClient::connect_tcp(*port, *addr).await?,
            ConnectionInfo::Unix { socket } => UhuraClient::connect_unix(socket).await?,
        };
        Ok(client)
    }
}
