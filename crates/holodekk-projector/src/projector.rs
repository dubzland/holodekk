use holodekk_core::engine::docker;
use holodekk_core::engine::{Image, ImageBuilder, ImageStore};

pub struct ProjectorBuilder<T: Image> {
    store: Option<Box<dyn ImageStore<Image = T>>>,
    builder: Option<Box<dyn ImageBuilder<Image = T>>>,
}

impl<T: Image> ProjectorBuilder<T> {
    fn new() -> Self {
        Default::default()
    }

    pub fn with_docker_store(self) -> Self
    where
        docker::Store: ImageStore<Image = T>,
    {
        let store = Box::new(docker::Store::new());
        Self {
            store: Some(store),
            ..self
        }
    }

    pub fn with_docker_builder(self) -> Self
    where
        docker::Builder: ImageBuilder<Image = T>,
    {
        let builder = Box::new(docker::Builder::new());
        // let store = Box::new(docker::Store::new()) as Box<dyn ImageStore<Image = DockerImage>>;

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

pub struct Projector<T: Image> {
    store: Box<dyn ImageStore<Image = T>>,
    builder: Box<dyn ImageBuilder<Image = T>>,
}

impl<T: Image> Projector<T> {
    fn new(
        store: Box<dyn ImageStore<Image = T>>,
        builder: Box<dyn ImageBuilder<Image = T>>,
    ) -> Self {
        Self { store, builder }
    }

    fn store(&self) -> &Box<dyn ImageStore<Image = T>> {
        &self.store
    }

    fn builder(&self) -> &Box<dyn ImageBuilder<Image = T>> {
        &self.builder
    }

    pub fn build() -> ProjectorBuilder<T> {
        ProjectorBuilder::new()
    }
}
