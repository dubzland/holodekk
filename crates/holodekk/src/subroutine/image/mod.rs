//! Subroutine images.
//!
//! Each [`Subroutine`][`crate::subroutine::entity::Entity`] running on the Holodekk is backed
//! by an "image" residing on disk.  These are currently just a directory containing the code that
//! makes up the subroutine.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::image;
use crate::subroutine;

/// Subroutine specific image id
pub type Id = image::Id;

/// Subroutine image
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Image {
    /// Image id
    pub id: Id,
    /// Image name
    pub name: image::Name,
    /// Path on disk to the image
    pub path: PathBuf,
    /// Kind of subroutine (ruby, node, etc)
    pub kind: subroutine::Kind,
}

impl Image {
    /// Constructs a new subroutine image.
    ///
    /// This does not actually perform any storage operations.
    pub fn new<P>(name: image::Name, path: P, kind: subroutine::Kind) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            id: Id::generate(&name),
            name,
            path: path.into(),
            kind,
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::subroutine::Kind;

    use super::*;

    #[fixture]
    pub fn mock_image() -> Image {
        Image::new(
            "test/sub".into(),
            "/tmp/holodekk/subroutines/test/sub",
            Kind::Ruby,
        )
    }
}

#[cfg(test)]
pub use fixtures::*;
