// use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::default::Default;

use async_trait::async_trait;

use bollard::image::ListImagesOptions;
use bollard::Docker;

use regex::Regex;

// use super::{ImageStore, Image, ImageTag, ImageKind};
use super::{DockerImage, DockerImageTag};
use crate::engine::{Engine, ImageBuilder, ImageKind, ImageStore};
use crate::errors::{Error, Result};

pub struct Service {
    prefix: String,
    client: bollard::Docker,
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
impl ImageBuilder<DockerImage, DockerImageTag> for Service {
    async fn build_subroutine(
        &self,
        _name: &str,
        _tag: &str,
        _data: &Vec<u8>,
    ) -> Result<DockerImage> {
        Ok(DockerImage::new("foo", ImageKind::Subroutine))
    }
}

impl Engine<DockerImage, DockerImageTag> for Service {}
