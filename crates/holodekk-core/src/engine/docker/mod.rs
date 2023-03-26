//! Tooling to interact with a [Docker](https://docker.com) daemon for container management.
//!
//! Utilizes the awesome [bollard](https://github.com/fussybeaver/bollard) crate.

mod build;
mod store;

use super::Engine;

pub(crate) const DOCKER_PREFIX: &str = "holodekk";

/// Necessary services for building, publishing, and executing containers on the Docker platform.
///
/// # Examples
///
/// ```rust
/// use holodekk::engine::docker::Docker;
///
/// let engine = Docker::new();
/// ```
pub struct Docker {
    client: bollard::Docker,
    prefix: String,
}

impl Default for Docker {
    fn default() -> Self {
        Self {
            client: bollard::Docker::connect_with_socket_defaults().unwrap(),
            prefix: DOCKER_PREFIX.to_string(),
        }
    }
}

impl Docker {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Engine for Docker {}
