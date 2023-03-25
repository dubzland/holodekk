// use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::default::Default;
use std::path::PathBuf;

use async_trait::async_trait;

use bollard::image::{BuildImageOptions, ListImagesOptions};
use bollard::Docker;

use futures_util::stream::StreamExt;

use regex::Regex;

use tar::Builder as TarBuilder;

// use super::{ImageStore, Image, ImageTag, ImageKind};
use super::{DockerImage, DockerImageTag};
use crate::engine::{Engine, Image, ImageBuilder, ImageKind, ImageStore};
use crate::errors::{Error, Result};
use crate::subroutine::Subroutine;

pub struct Service {
    prefix: String,
    client: bollard::Docker,
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Service {
    pub fn new() -> Self {
        let client = Docker::connect_with_socket_defaults().unwrap();
        Self {
            prefix: "holodekk".to_string(),
            client,
        }
    }

    fn image_from_tag(&self, tag_str: &str, id: &str) -> Result<Option<DockerImage>> {
        let re = Regex::new(
            format!(
                r"{}/(subroutine|service|application)/([^:]*):(.*)",
                self.prefix
            )
            .as_str(),
        )
        .unwrap();

        if let Some(matches) = re.captures(tag_str) {
            let kind = match matches.get(1).unwrap().as_str() {
                "application" => ImageKind::Application,
                "service" => ImageKind::Service,
                "subroutine" => ImageKind::Subroutine,
                &_ => panic!("matched, but not matched"),
            };
            let name = matches.get(2).unwrap().as_str();
            let tag = matches.get(3).unwrap().as_str();

            let image = DockerImage::new(name, kind);
            image.add_tag(tag, id);
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    async fn images(&self) -> Result<Vec<DockerImage>> {
        let mut filters = HashMap::new();
        filters.insert("dangling", vec!["false"]);
        let options = ListImagesOptions {
            filters,
            ..Default::default()
        };

        let mut images = vec![];

        match self.client.list_images(Some(options)).await {
            Ok(docker_images) => {
                for image in docker_images {
                    for tag in &image.repo_tags {
                        if let Some(img) = self.image_from_tag(tag, &image.id)? {
                            images.push(img);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error received from docker: {}", e);
                // Convert the bollard erro to ours.
                return Err(Error::BollardError(e));
            }
        };
        Ok(images)
    }
}

#[async_trait]
impl ImageStore<DockerImage, DockerImageTag> for Service {
    /// Retrieve a list of subroutine images from Docker.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekk_core::Result;
    /// use holodekk_core::engine::{docker, ImageStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let docker = docker::Service::new();
    ///     let images = docker.subroutine_images().await?;
    ///     Ok(())
    /// }
    /// ```
    async fn subroutine_images(&self) -> Result<Vec<DockerImage>> {
        let images = self.images().await?;
        let (sub_images, _): (Vec<DockerImage>, Vec<DockerImage>) = images
            .into_iter()
            .partition(|i| i.kind().eq(&ImageKind::Subroutine));
        Ok(sub_images)
    }

    /// Retrieve a list of application images from Docker.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekk_core::Result;
    /// use holodekk_core::engine::{docker, ImageStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let docker = docker::Service::new();
    ///     let images = docker.application_images().await?;
    ///     Ok(())
    /// }
    /// ```
    async fn application_images(&self) -> Result<Vec<DockerImage>> {
        let images = self.images().await?;
        let (sub_images, _): (Vec<DockerImage>, Vec<DockerImage>) = images
            .into_iter()
            .partition(|i| i.kind().eq(&ImageKind::Application));
        Ok(sub_images)
    }

    /// Determine whether a given application image exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekk_core::Result;
    /// use holodekk_core::engine::{docker, ImageStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let docker = docker::Service::new();
    ///     let images = docker.application_image_exists().await?;
    ///     Ok(())
    /// }
    /// ```
    async fn application_image_exists(&self, subroutine: &Subroutine) -> Result<bool> {
        let images = self.application_images().await?;
        Ok(images.iter().any(|i| i.name().eq(&subroutine.name)))
        // let (sub_images, _): (Vec<DockerImage>, Vec<DockerImage>) = images
        //     .into_iter()
        //     .partition(|i| i.kind().eq(&ImageKind::Application));
        // Ok(sub_images)
    }
}

#[async_trait]
impl ImageBuilder<DockerImage, DockerImageTag> for Service {
    async fn build_subroutine(&self, name: &str, tag: &str, data: Vec<u8>) -> Result<DockerImage> {
        let options = BuildImageOptions {
            t: format!("{}:{}", name, tag),
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
            t: format!("holodekk/application/{}:latest", subroutine.name),
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

impl Engine<DockerImage, DockerImageTag> for Service {}

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
