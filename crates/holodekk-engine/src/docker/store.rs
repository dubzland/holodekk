use std::collections::HashMap;

use async_trait::async_trait;

use bollard::image::ListImagesOptions;

use regex::Regex;

use crate::docker::Docker;
use crate::{Image, ImageKind, Result, Store};

pub(crate) struct ImageStore<'a> {
    client: &'a bollard::Docker,
    prefix: &'a str,
}

impl<'a> ImageStore<'a> {
    fn new(client: &'a bollard::Docker, prefix: &'a str) -> Self {
        Self { client, prefix }
    }

    fn image_from_tag(&self, tag_str: &str, id: &str) -> Result<Option<Image>> {
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

            let image = Image::new(name, kind);
            image.add_tag(tag, id);
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    async fn fetch_images(&self) -> Result<Vec<Image>> {
        let mut filters = HashMap::new();
        filters.insert("dangling", vec!["false"]);
        let options = ListImagesOptions {
            filters,
            ..Default::default()
        };

        let docker_images = self.client.list_images(Some(options)).await?;
        // match self.client.list_images(Some(options)).await {
        //     Ok(docker_images) => {
        let mut images = vec![];
        for image in docker_images {
            for tag in &image.repo_tags {
                if let Some(img) = self.image_from_tag(tag, &image.id)? {
                    images.push(img);
                }
            }
        }
        Ok(images)
        // }
        // Err(e) => {
        //     // Convert the bollard erro to ours.
        //     Err(Error::BollardError(e))
        // }
        // }
    }
}

#[async_trait]
impl<'a> Store for ImageStore<'a> {
    async fn images(&self, kind: ImageKind) -> crate::Result<Vec<Image>> {
        let all_images = self.fetch_images().await?;
        let (images, _): (Vec<Image>, Vec<Image>) =
            all_images.into_iter().partition(|i| i.kind().eq(&kind));
        Ok(images)
    }

    async fn image_exists(&self, kind: ImageKind, name: &str) -> crate::Result<bool> {
        let images = match kind {
            ImageKind::Application => self.images(ImageKind::Application),
            ImageKind::Subroutine => self.images(ImageKind::Subroutine),
            ImageKind::Service => return Ok(false),
        };
        Ok(images.await.unwrap().iter().any(|i| i.name().eq(name)))
    }
}

#[async_trait]
impl Store for Docker {
    /// Retrieve a list of images from Docker.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekk_engine::{docker, ImageKind, Store};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let engine = docker::Docker::connect();
    /// let images = engine.images(ImageKind::Subroutine).await.unwrap();
    /// # }
    /// ```
    async fn images(&self, kind: ImageKind) -> crate::Result<Vec<Image>> {
        ImageStore::new(&self.client, &self.prefix)
            .images(kind)
            .await
    }

    /// Determine whether a given image exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use holodekk_engine::{docker, ImageKind, Store};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let engine = docker::Docker::connect();
    /// if engine.image_exists(ImageKind::Application, "acme/widget-api").await.unwrap() {
    ///     println!("Image exists!");
    /// }
    /// # }
    /// ```
    async fn image_exists(&self, kind: ImageKind, name: &str) -> crate::Result<bool> {
        ImageStore::new(&self.client, &self.prefix)
            .image_exists(kind, name)
            .await
    }
}
