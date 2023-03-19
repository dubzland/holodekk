use std::collections::HashMap;
use std::default::Default;

use async_trait::async_trait;

use bollard::Docker;
use bollard::image::ListImagesOptions;

use bytes::Bytes;

use regex::Regex;

use super::{Engine, ImageDef, ImageDefKind, Result};

pub struct DockerEngine {
    prefix: String,
    client: bollard::Docker,
}

impl DockerEngine {
    pub fn new() -> Self {
        let client = Docker::connect_with_socket_defaults().unwrap();
        Self { prefix: "holodekk".to_string(), client }
    }
}

#[async_trait]
impl Engine for DockerEngine {

    /// Retrieve a list of images from Docker for the specified type.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::engine::{docker::DockerEngine, Engine, ImageDefKind};
    ///
    /// let engine = DockerEngine::new();
    /// let images = engine.list_images(ImageDefKind::Subroutine);
    /// ```
    async fn list_images(&self, kind: ImageDefKind) -> Vec<ImageDef> {
        let sub_re = Regex::new(
            format!(r"{}/subroutine/([^:]*):(.*)", self.prefix).as_str()
        ).unwrap();
        let srv_re = Regex::new(
            format!(r"{}/service/([^:]*):(.*)", self.prefix).as_str()
        ).unwrap();
        let app_re = Regex::new(
            format!(r"{}/application/([^:]*):(.*)", self.prefix).as_str()
        ).unwrap();

        let re = match kind {
            ImageDefKind::Subroutine => &sub_re,
            ImageDefKind::Service => &srv_re,
            ImageDefKind::Application => &app_re,
        };

        let mut filters = HashMap::new();
        filters.insert("dangling", vec!["false"]);
        let options = ListImagesOptions {
            filters,
            ..Default::default()
        };

        let mut image_defs = vec![];

        for image in self.client.list_images(Some(options)).await.unwrap() {
            let repo_tags = &image.repo_tags;
            for tag in repo_tags {
                if re.is_match(tag) {
                    if let Some(matches) = re.captures(tag) {
                        image_defs.push(ImageDef {
                            name: matches.get(1).unwrap().as_str().to_string(),
                            tag: matches.get(2).unwrap().as_str().to_string(),
                            id: Some(image.id.to_owned()),
                            kind: kind.to_owned(),
                        });
                    }
                }
            };
        };
        image_defs
    }

    async fn build_image(&self, def: &mut ImageDef, rootfs: &Bytes) -> Result<()> {
        Ok(())
    }
}
