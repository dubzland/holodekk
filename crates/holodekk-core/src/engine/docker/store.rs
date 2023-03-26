use std::collections::HashMap;

use async_trait::async_trait;

use bollard::image::ListImagesOptions;

use regex::Regex;

use crate::engine::{Image, ImageKind, ImageStore};
use crate::errors::{Error, Result};

use super::DockerImage;

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
impl ImageStore for Store {
    type Image = DockerImage;

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
    ///     let store = docker::Store::new();
    ///     let images = store.subroutine_images().await?;
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
    ///     let store = docker::Store::new();
    ///     let images = store.application_images().await?;
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

    /// Determine whether a given image exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekk_core::Result;
    /// use holodekk_core::engine::{docker, ImageKind, ImageStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let store = docker::Store::new();
    ///     if store.image_exists(ImageKind::Application, "acme/widget-api").unwrap() {
    ///         println!("Image exists!");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn image_exists(&self, kind: ImageKind, name: &str) -> Result<bool> {
        let images = match kind {
            ImageKind::Application => self.application_images(),
            ImageKind::Subroutine => self.subroutine_images(),
            ImageKind::Service => return Ok(false),
        };
        Ok(images.await.unwrap().iter().any(|i| i.name().eq(name)))
    }
}
