use holodekk::engine::{docker::Docker, Engine};

use super::Projector;

#[derive(Default)]
pub struct ProjectorBuilder {
    namespace: Option<String>,
    engine: Option<Box<dyn Engine>>,
}

impl ProjectorBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn for_namespace(self, namespace: &str) -> Self {
        Self {
            namespace: Some(namespace.to_string()),
            ..self
        }
    }

    pub fn with_docker_engine(self) -> Self {
        Self {
            engine: Some(Box::new(Docker::new())),
            ..self
        }
    }

    pub fn build(self) -> Projector {
        Projector::new(self.namespace.unwrap().as_str(), self.engine.unwrap())
    }
}
