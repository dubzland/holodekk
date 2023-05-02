use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::enums::SubroutineKind;

use super::{ImageId, ImageName};

pub type SubroutineImageId = ImageId;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineImage {
    pub id: SubroutineImageId,
    pub name: ImageName,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

impl SubroutineImage {
    pub fn new<P>(name: ImageName, path: P, kind: SubroutineKind) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            id: SubroutineImageId::generate(&name),
            name,
            path: path.into(),
            kind,
        }
    }
}
