use std::default::Default;

use async_trait::async_trait;

use bollard::image::BuildImageOptions;

use futures_util::stream::StreamExt;

use super::DockerImage;
use crate::engine::{ImageBuilder, ImageKind};
use crate::holodekk::Application;

pub struct DockerImageBuilder {
    prefix: String,
    client: bollard::Docker,
}

impl Default for DockerImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerImageBuilder {
    pub fn new() -> Self {
        let client = bollard::Docker::connect_with_socket_defaults().unwrap();
        Self {
            prefix: super::DOCKER_PREFIX.to_string(),
            client,
        }
    }
}

#[async_trait]
impl ImageBuilder for DockerImageBuilder {
    type Image = DockerImage;

    async fn build_subroutine(
        &self,
        name: &str,
        tag: &str,
        data: Vec<u8>,
    ) -> crate::Result<DockerImage> {
        let options = BuildImageOptions {
            t: format!("{}/subroutine/{}:{}", self.prefix, name, tag),
            q: true,
            ..Default::default()
        };
        let mut image_stream = self.client.build_image(options, None, Some(data.into()));
        while let Some(msg) = image_stream.next().await {
            println!("msg: {:?}", msg);
        }
        Ok(DockerImage::new("foo", ImageKind::Subroutine))
    }

    async fn build_application(
        &self,
        application: &Application<DockerImage>,
        // bytes: &'static [u8],
        bytes: Vec<u8>,
    ) -> crate::Result<DockerImage> {
        let options = BuildImageOptions {
            t: format!("{}/application/{}:latest", self.prefix, application.name()),
            ..Default::default()
        };

        let mut image_stream = self.client.build_image(options, None, Some(bytes.into()));
        while let Some(msg) = image_stream.next().await {
            match msg {
                Ok(build_info) => {
                    if let Some(stream) = build_info.stream {
                        print!("{}", stream);
                    }
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
        Ok(DockerImage::new("foo", ImageKind::Subroutine))
    }
}
