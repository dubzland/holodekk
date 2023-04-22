use std::default::Default;

use async_trait::async_trait;

use bollard::image::BuildImageOptions;

use futures_util::stream::StreamExt;

use super::Docker;
use crate::engines::{Build, Image, ImageKind, Result};

pub(crate) struct ImageBuilder<'a> {
    client: &'a bollard::Docker,
    prefix: &'a str,
}

impl<'a> ImageBuilder<'a> {
    fn new(client: &'a bollard::Docker, prefix: &'a str) -> Self {
        Self { client, prefix }
    }
}

#[async_trait]
impl<'a> Build for ImageBuilder<'a> {
    async fn build(
        &self,
        kind: ImageKind,
        name: &str,
        tag: &str,
        data: Vec<u8>,
        definition: Option<&str>,
    ) -> Result<Image> {
        let short_name = format!("{}:{}", name, tag);
        let full_name = match kind {
            ImageKind::Application => format!("{}/application/{}", self.prefix, short_name),
            ImageKind::Service => format!("{}/service/{}", self.prefix, short_name),
            ImageKind::Subroutine => format!("{}/subroutine/{}", self.prefix, short_name),
        };

        let options = BuildImageOptions {
            dockerfile: definition.unwrap_or(""),
            t: &full_name,
            q: true,
            ..Default::default()
        };
        let mut image_stream = self.client.build_image(options, None, Some(data.into()));
        while let Some(msg) = image_stream.next().await {
            println!("msg: {:?}", msg);
        }
        Ok(Image::new(&short_name, kind))
    }
}

#[async_trait]
impl Build for Docker {
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tar::{Builder, Header};
    /// # use holodekk::engines::{docker::Docker, Build, ImageKind};
    /// # async fn create_archive() -> Vec<u8> {
    /// #     let dockerfile = r#"FROM scratch"#;
    /// #     let mut header = Header::new_gnu();
    /// #     let bytes = dockerfile.as_bytes().to_vec();
    /// #     header.set_size(bytes.len().try_into().unwrap());
    /// #     header.set_cksum();
    /// #     let mut data = Vec::default();
    /// #     let mut archive = Builder::new(bytes);
    /// #     archive.append_data(&mut header, "Dockerfile.v1", dockerfile.as_bytes()).unwrap();
    /// #     data
    /// # }
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let data = create_archive().await;
    /// # let engine = Docker::connect();
    /// let image = engine.build(
    ///     ImageKind::Application,
    ///     "testing",
    ///     "latest",
    ///     data,
    ///     Some("Dockerfile.v1")
    /// ).await.unwrap();
    /// # }
    /// ```
    async fn build(
        &self,
        kind: ImageKind,
        name: &str,
        tag: &str,
        data: Vec<u8>,
        definition: Option<&str>,
    ) -> Result<Image> {
        ImageBuilder::new(&self.client, &self.prefix)
            .build(kind, name, tag, data, definition)
            .await
    }
}
