mod image;
mod image_builder;
mod image_store;

pub use image::{DockerImage, DockerImageTag};
pub use image_builder::DockerImageBuilder;
pub use image_store::DockerImageStore;

use async_trait::async_trait;

use super::{Engine, ImageBuilder, ImageStore};

pub(crate) const DOCKER_PREFIX: &str = "holodekk";

pub struct Docker {
    image_builder: DockerImageBuilder,
    image_store: DockerImageStore,
}

#[async_trait]
impl Engine<DockerImage> for Docker {
    async fn image_builder(&self) -> crate::Result<&dyn ImageBuilder<Image = DockerImage>> {
        Ok(&self.image_builder)
    }

    async fn image_store(&self) -> crate::Result<&dyn ImageStore<Image = DockerImage>> {
        Ok(&self.image_store)
    }
}
