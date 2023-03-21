use std::cell::{Ref, RefCell};

use serde::Serialize;

use crate::engine::{Image, ImageKind, ImageTag};

/// Represents a pairing between a human readable tag and the internal id.
///
/// # Examples
///
/// ```rust
/// use holodekk_core::engine::docker::DockerImageTag;
///
/// let tag = DockerImageTag::new("acme/widgets", "sha256:5a8a49...e1bdc6");
/// ```
#[derive(Debug, Serialize)]
pub struct DockerImageTag {
    name: String,
    _id: String,
}

impl DockerImageTag {
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_string(),
            _id: id.to_string(),
        }
    }
}

impl ImageTag for DockerImageTag {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize)]
pub struct DockerImage {
    name: String,
    tags: RefCell<Vec<DockerImageTag>>,
    kind: ImageKind,
}

impl DockerImage {
    pub fn new(name: &str, kind: ImageKind) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
            tags: RefCell::new(vec![]),
        }
    }

    pub fn add_tag(&self, tag: &str, engine_id: &str) {
        self.tags
            .borrow_mut()
            .push(DockerImageTag::new(tag, engine_id));
    }
}

impl Image<DockerImageTag> for DockerImage {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &ImageKind {
        &self.kind
    }

    fn tags(&self) -> Ref<'_, Vec<DockerImageTag>> {
        self.tags.borrow()
    }
}
