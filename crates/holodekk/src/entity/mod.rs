use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref ENTITY_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(String);

impl Id {
    pub fn generate() -> Self {
        Self(id::generate())
    }
}

impl FromStr for Id {
    type Err = id::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if ENTITY_ID_RE.is_match(s) {
            Ok(Id(s.to_string()))
        } else {
            Err(id::Error::Format(s.to_string()))
        }
    }
}

impl TryFrom<String> for Id {
    type Error = id::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<Id> for String {
    fn from(id: Id) -> String {
        id.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Id
where
    T: ?Sized,
    <Id as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

pub mod id {
    use rand::RngCore;

    pub fn generate() -> String {
        let mut bytes: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("Invalid EntityId format")]
        Format(String),
    }
}
pub mod repository;
pub use repository::Repository;
pub mod service;
