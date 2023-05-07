//! Repository storage representation for a running scene instance.

pub mod api;
mod name;
pub use name::*;
pub mod repository;
pub use repository::Repository;
pub mod service;
pub use service::Service;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::entity;

/// Scene specific entity id (data storage key)
pub type Id = entity::Id;

/// Current status of a given scene
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Status {
    /// Unknown
    Unknown,
    /// Created (but not running)
    Created,
    /// Running (but not synchronized)
    Starting(i32),
    /// Fully operational
    Running(i32),
    /// Terminated via command
    Stopped,
    /// Stopped without being requested
    Crashed,
}

/// Data storage entity for a given scene
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct Entity {
    /// Scene key
    pub id: Id,
    /// Name assigned by the end user
    pub name: Name,
    /// Current status of this scene (projector)
    pub status: Status,
    /// Timestamp for the scene's repository record
    pub created_at: Option<NaiveDateTime>,
    /// Last time this scene's repository record was updated
    pub updated_at: Option<NaiveDateTime>,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: Id::generate(),
            name: "".into(),
            status: Status::Unknown,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Entity {
    /// Shorhand for creating a new scene entity
    #[must_use]
    pub fn new(name: Name) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod fixtures {
    use chrono::Utc;
    use rstest::fixture;

    use super::*;

    #[fixture]
    #[cfg(test)]
    pub fn mock_entity() -> Entity {
        let mut scene = Entity::new("test".into());
        scene.created_at = Some(Utc::now().naive_utc());
        scene
    }
}

#[cfg(test)]
pub use fixtures::*;
