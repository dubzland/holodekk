use std::default::Default;
use std::path::PathBuf;

use async_trait::async_trait;

use bollard::image::BuildImageOptions;
use bollard::Docker;

use futures_util::stream::StreamExt;

use tar::Builder as TarBuilder;

use super::{DockerImage, DockerImageTag};
use crate::engine::{ImageBuilder, ImageKind};
use crate::errors::Result;
use crate::subroutine::Subroutine;

pub struct Builder {
    prefix: String,
    client: bollard::Docker,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        let client = Docker::connect_with_socket_defaults().unwrap();
        Self {
            prefix: super::DOCKER_PREFIX.to_string(),
            client,
        }
    }
}

#[async_trait]
impl ImageBuilder<DockerImage, DockerImageTag> for Builder {
    async fn build_subroutine(&self, name: &str, tag: &str, data: Vec<u8>) -> Result<DockerImage> {
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

    async fn build_application(&self, subroutine: &Subroutine) -> Result<DockerImage> {
        let options = BuildImageOptions {
            t: format!("{}/application/{}:latest", self.prefix, subroutine.name),
            ..Default::default()
        };

        let context = PathBuf::from(&subroutine.container.context);

        let mut bytes = Vec::default();
        create_archive(&context, &mut bytes).unwrap();

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

fn create_archive<T: std::io::Write>(
    context: &PathBuf,
    // dockerfile: &str,
    target: T,
) -> std::io::Result<()> {
    let mut archive: TarBuilder<T> = TarBuilder::new(target);

    // let mut header = Header::new_gnu();
    // let bytes = dockerfile.as_bytes().to_vec();
    // header.set_size(bytes.len().try_into().unwrap());
    // header.set_cksum();

    // archive
    //     .append_data(&mut header, "Dockerfile", dockerfile.as_bytes())
    //     .unwrap();

    archive.append_dir_all("", context)?;

    Ok(())
}
