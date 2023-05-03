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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Status {
    Unknown,
    Created,
    Starting(i32),
    Running(i32),
    Stopped,
    Crashed,
}
pub type Id = entity::Id;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct Entity {
    pub id: Id,
    pub name: Name,
    pub status: Status,
    pub created_at: Option<NaiveDateTime>,
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
