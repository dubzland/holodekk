use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::image;
use crate::subroutine;

pub type Id = image::Id;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Image {
    pub id: Id,
    pub name: image::Name,
    pub path: PathBuf,
    pub kind: subroutine::Kind,
}

impl Image {
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
