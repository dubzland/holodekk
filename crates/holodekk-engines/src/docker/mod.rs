//! Tooling to interact with a [Docker](https://docker.com) daemon for container management.
//!
//! Utilizes the awesome [bollard](https://github.com/fussybeaver/bollard) crate.
mod build;
mod store;

use log::debug;

use url::Url;

use crate::{Engine, Identity};

pub(crate) const DOCKER_PREFIX: &str = "holodekk";

/// Necessary services for building, publishing, and executing containers on the Docker platform.
///
/// # Examples
///
/// ```rust,no_run
/// use holodekk_common::engines::docker::Docker;
///
/// let engine = Docker::connect();
/// ```
pub struct Docker {
    client: bollard::Docker,
    prefix: String,
}

impl Default for Docker {
    fn default() -> Self {
        Self::connect_local()
    }
}

impl Docker {
    fn new(client: bollard::Docker) -> Self {
        Self {
            client,
            prefix: DOCKER_PREFIX.into(),
        }
    }

    pub fn connect_local() -> Self {
        Self::new(bollard::Docker::connect_with_socket_defaults().unwrap())
    }

    pub fn connect_http() -> Self {
        Self::new(bollard::Docker::connect_with_http_defaults().unwrap())
    }

    pub fn connect_https() -> Self {
        Self::new(bollard::Docker::connect_with_ssl_defaults().unwrap())
    }

    pub fn connect() -> Self {
        if let Ok(url) = std::env::var("DOCKER_HOST") {
            let docker_url = Url::parse(&url).unwrap();
            match docker_url.scheme() {
                "http" => {
                    debug!("Connecting to Docker via HTTP: {}", url);
                    Self::connect_http()
                }
                "https" => {
                    debug!("Connecting to Docker via HTTPS: {}", url);
                    Self::connect_https()
                }
                &_ => panic!("Invalid DOCKER_HOST specified: {}", url),
            }
        } else {
            debug!("Connecting via local socket");
            Self::connect_local()
        }
    }
}

impl Identity for Docker {
    fn name(&self) -> &'static str {
        "docker"
    }
}

impl Engine for Docker {}
