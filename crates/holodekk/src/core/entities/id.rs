use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use lazy_static::lazy_static;
use rand::RngCore;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub fn generate_id() -> String {
    let mut bytes: [u8; 32] = [0; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

lazy_static! {
    static ref ENTITY_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum EntityIdError {
    #[error("Invalid EntityId format")]
    Format(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct EntityId(String);

impl EntityId {
    pub fn generate() -> Self {
        Self(generate_id())
    }
}

impl FromStr for EntityId {
    type Err = EntityIdError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if ENTITY_ID_RE.is_match(s) {
            Ok(EntityId(s.to_string()))
        } else {
            Err(EntityIdError::Format(s.to_string()))
        }
    }
}

impl TryFrom<String> for EntityId {
    type Error = EntityIdError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for EntityId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for EntityId
where
    T: ?Sized,
    <EntityId as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}
