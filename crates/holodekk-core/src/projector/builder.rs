use crate::engine::docker::{DockerImageBuilder, DockerImageStore};
use crate::engine::{Image, ImageBuilder, ImageStore};

use super::Projector;

pub struct ProjectorBuilder<T: Image> {
    store: Option<Box<dyn ImageStore<Image = T>>>,
    builder: Option<Box<dyn ImageBuilder<Image = T>>>,
}

impl<T: Image> ProjectorBuilder<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_docker_store(self) -> Self
    where
        DockerImageStore: ImageStore<Image = T>,
    {
        let store = Box::new(DockerImageStore::new());
        Self {
            store: Some(store),
            ..self
        }
    }

    pub fn with_docker_builder(self) -> Self
    where
        DockerImageBuilder: ImageBuilder<Image = T>,
    {
        let builder = Box::new(DockerImageBuilder::new());

        Self {
            builder: Some(builder),
            ..self
        }
    }

    pub fn build(self) -> Projector<T> {
        let store = self.store.unwrap();
        let builder = self.builder.unwrap();
        Projector::new(store, builder)
    }
}

impl<T: Image> Default for ProjectorBuilder<T> {
    fn default() -> Self {
        ProjectorBuilder {
            store: None,
            builder: None,
        }
    }
}
