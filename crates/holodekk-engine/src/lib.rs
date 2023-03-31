//! Interface to underlying container infrastructure.
//!
//! The [Engine] encapsulates all aspects of container management, from building
//! and pulling images to running container instances.
//!
//! Currently available engines:
//!  - [Docker](docker::Docker)
//!
//! To add a new engine type to the system, you must first implement the actual
//! [Image] type, and its associated [ImageTag].  This allows the image itself
//! to be represented in a generic fashion, while still allowing the engine the
//! freedom to handle the implementation in whatever way is required.
//!
//! Images are categorized according to the type of container that will be created.  See
//! [ImageKind] for more information.
//!
//! Then, the following traits must be implemented:
//!  - [Store]
//!  - [Build]
//!  - [Engine]
//!
//! Finally, they must all be tied together into an actual [Engine].  See
//! [Docker](docker::Docker) for an example of a fully-functional [Engine].
pub mod docker;
mod errors;
pub use errors::{Error, Result};

use std::sync::{RwLock, RwLockReadGuard};

use async_trait::async_trait;

use serde::{Deserialize, Serialize};

/// The actual type of image being managed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ImageKind {
    /// Subroutine images are meant to be run as [Subroutine](crate::subroutine::Subroutine)s
    /// directly by the [Projector](crate::projector::Projector).
    Subroutine,
    /// Services represent anything provided as a dependency, such as:
    ///  - Databases
    ///  - Caches
    ///  - Proxies
    Service,
    /// The actual [Application](crate::subroutine::Application) being managed by
    /// a [Subroutine](crate::subroutine::Subroutine).
    Application,
}

/// Representation of the underlying image store's tag.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageTag {
    name: String,
    id: String,
}

impl ImageTag {
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
        }
    }

    /// String representing the tag portion of this image's id within the system.
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Represents a single image within the engine.
#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    name: String,
    kind: ImageKind,
    tags: RwLock<Vec<ImageTag>>,
}

impl Image {
    pub fn new(name: &str, kind: ImageKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
            tags: RwLock::new(vec![]),
        }
    }

    pub fn add_tag(&self, name: &str, id: &str) {
        self.tags.write().unwrap().push(ImageTag::new(name, id));
    }

    /// String representing the name portion of this image's id within the system[^note].
    ///
    /// [^note]: The actual name visible in the engine storage backend might not match.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &ImageKind {
        &self.kind
    }

    pub fn tags(&self) -> RwLockReadGuard<'_, Vec<ImageTag>> {
        self.tags.read().unwrap()
    }
}

/// Trait implemented by engines to provide container image build.
///
/// See [Docker](crate::engine::docker::Docker) for examples.
#[async_trait]
pub trait Build {
    /// Build a container image using the supplied context data.
    ///
    /// # Arguments
    ///
    /// * `kind` = The type of image to build
    /// * `name` = Name within the Holodekk to assign to this image
    /// * `tag` = Tag to apply to this image
    /// * `data` = Tar data (optionally compressed) used as the context
    /// * `definition` = Relative path to the Dockerfile within the context
    async fn build(
        &self,
        kind: ImageKind,
        name: &str,
        tag: &str,
        data: Vec<u8>,
        definition: Option<&str>,
    ) -> crate::Result<Image>;
}

/// Trait implemented by engines to provide image management capabilities.
///
/// See [Docker](crate::engine::docker::Docker) for examples.
#[async_trait]
pub trait Store {
    async fn images(&self, kind: ImageKind) -> crate::Result<Vec<Image>>;
    async fn image_exists(&self, kind: ImageKind, name: &str) -> crate::Result<bool>;
}

pub trait Identity {
    fn name(&self) -> &'static str;
}

pub trait Engine: Identity + Build + Store {}
