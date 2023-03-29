use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

use tonic::transport::Channel;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub struct ProjectorClientBuilder {}

impl ProjectorClientBuilder {
    pub async fn connect_tcp(
        &self,
        port: u16,
        addr: Option<&str>,
    ) -> crate::Result<ProjectorClient> {
        let connect_address: String;

        if let Some(addr) = addr {
            connect_address = format!("http://{}:{}", addr, port);
        } else {
            connect_address = format!("http://[::1]:{}", port);
        }

        let client = GreeterClient::connect(connect_address).await?;

        Ok(ProjectorClient { client })
    }
}

pub struct ProjectorClient {
    client: GreeterClient<Channel>,
}

impl ProjectorClient {
    pub fn build() -> ProjectorClientBuilder {
        ProjectorClientBuilder {}
    }

    pub async fn say_hello(&self, name: &str) -> crate::Result<String> {
        let request = tonic::Request::new(HelloRequest { name: name.into() });

        // let mut client = GreeterClient::connect(address).await?;
        let mut client = self.client.clone();

        let response = client.say_hello(request).await?;

        Ok(response.into_inner().message)
    }
}
