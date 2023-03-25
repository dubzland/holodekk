use std::collections::HashMap;

use async_trait::async_trait;

use bollard::image::ListImagesOptions;

use regex::Regex;

use crate::engine::{Image, ImageKind, ImageStore};
use crate::errors::{Error, Result};
use crate::subroutine::Subroutine;

use super::{DockerImage, DockerImageTag};

pub struct Store {
    prefix: String,
    client: bollard::Docker,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let client = bollard::Docker::connect_with_socket_defaults().unwrap();
        Self {
            prefix: super::DOCKER_PREFIX.to_string(),
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

        match self.client.list_images(Some(options)).await {
            Ok(docker_images) => {
                let mut images = vec![];
                for image in docker_images {
                    for tag in &image.repo_tags {
                        if let Some(img) = self.image_from_tag(tag, &image.id)? {
                            images.push(img);
                        }
                    }
                }
                Ok(images)
            }
            Err(e) => {
                // Convert the bollard erro to ours.
                Err(Error::BollardError(e))
            }
        }
    }
}

#[async_trait]
impl ImageStore<DockerImage, DockerImageTag> for Store {
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
