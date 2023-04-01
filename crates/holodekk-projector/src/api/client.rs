use std::net::Ipv4Addr;
use tonic::transport::Channel;

use super::proto::applications::ApplicationsClient;
use super::proto::entities::Empty;

pub struct ProjectorClientBuilder {}

impl ProjectorClientBuilder {
    pub async fn connect_tcp(&self, port: u16, addr: Ipv4Addr) -> crate::Result<ProjectorClient> {
        let connect_address = format!("http://{}:{}", addr, port);

        let client = ApplicationsClient::connect(connect_address).await?;

        Ok(ProjectorClient { client })
    }
}

pub struct ProjectorClient {
    client: ApplicationsClient<Channel>,
}

impl ProjectorClient {
    pub fn build() -> ProjectorClientBuilder {
        ProjectorClientBuilder {}
    }

    pub async fn list(&self) -> crate::Result<String> {
        let request = tonic::Request::new(Empty {});

        let mut client = self.client.clone();

        let response = client.list(request).await?;

        Ok(response.into_inner().message)
    }
}
