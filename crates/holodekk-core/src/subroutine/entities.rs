use std::fmt;

use serde::{Deserialize, Serialize};

use crate::engine::docker;
use crate::engine::ImageStore;
use crate::errors::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    pub context: String,
    pub dockerfile: String,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.context, self.dockerfile)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subroutine {
    pub name: String,
    pub container: Container,
}

impl Subroutine {
    pub async fn container_image_exists(&self) -> Result<bool> {
        let docker = docker::Service::new();
        docker.application_image_exists(self).await
    }
}

impl fmt::Display for Subroutine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
