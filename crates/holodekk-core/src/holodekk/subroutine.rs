use std::fmt;

use serde::{Deserialize, Serialize};

use crate::engine::Image;

#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    context: String,
    dockerfile: String,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.context, self.dockerfile)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Application<T: Image> {
    name: String,
    image: Option<Box<T>>,
}

impl<T: Image> Default for Application<T> {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            image: None,
        }
    }
}

impl<T: Image> Application<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_image(self, image: Box<T>) -> Self {
        Self {
            image: Some(image),
            ..self
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subroutine<T: Image> {
    name: String,
    application: Application<T>,
}

impl<T: Image> Subroutine<T> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn application(&self) -> &Application<T> {
        &self.application
    }

    pub fn from_manifest(manifest: &SubroutineManifest) -> Subroutine<T> {
        let application = Application {
            name: manifest.name().to_string(),
            image: None,
        };

        Self {
            name: manifest.name().to_string(),
            application,
        }
    }
}

impl<T: Image> fmt::Display for Subroutine<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContainerFromContext {
    pub context: String,
    pub dockerfile: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContainerManifest {
    FromContext { context: String, dockerfile: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubroutineManifest {
    name: String,
    container: ContainerManifest,
}

impl SubroutineManifest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn container(&self) -> &ContainerManifest {
        &self.container
    }
}
