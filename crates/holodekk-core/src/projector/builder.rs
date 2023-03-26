use crate::engine::docker::Docker;
use crate::engine::Engine;

use super::Projector;

#[derive(Default)]
pub struct ProjectorBuilder {
    engine: Option<Box<dyn Engine>>,
}

impl ProjectorBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_docker_engine(self) -> Self {
        Self {
            engine: Some(Box::new(Docker::new())),
        }
    }

    pub fn build(self) -> Projector {
        let engine = self.engine.unwrap();
        Projector::new(engine)
    }
}
