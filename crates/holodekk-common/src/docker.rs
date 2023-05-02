use actix_web::web::Bytes;

use bollard::image::BuildImageOptions;
use bollard::Docker;

use tokio_stream::{Stream, StreamExt};

pub struct Client {
    docker: Docker,
}

impl Client {
    pub fn new() -> Self {
        Self {
            docker: Docker::connect_with_socket_defaults().unwrap(),
        }
    }

    pub fn build(
        self,
        tag: &str,
        data: Bytes,
    ) -> impl Stream<Item = Result<Bytes, bollard::errors::Error>> {
        let options = BuildImageOptions::<String> {
            dockerfile: "Dockerfile".to_string(),
            t: tag.to_string(),
            q: true,
            ..Default::default()
        };
        async_stream::stream! {
            let mut data_stream = self.docker.build_image(options, None, Some(data.into()));
            while let Some(msg) = data_stream.next().await {
                match msg {
                    Ok(data) => {
                        println!("Received a data packet.");
                        println!("{:?}", data);
                    },
                    Err(err) => {
                        println!("Received an error.");
                        println!("{:?}", err);
                    }
                }
                yield Ok(Bytes::from(""));
            }
        }
    }
}
