mod builder;
mod image;
mod store;

pub use builder::Builder;
pub use image::{DockerImage, DockerImageTag};
pub use store::Store;

pub(crate) const DOCKER_PREFIX: &str = "holodekk";
