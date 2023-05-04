//! Repository-backed storage entity.
//!
//! Entities represent the manageable resources within the `Holodekk`.
use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref ENTITY_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

/// Newtype unique identifier for an `Entity`
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(String);

impl Id {
    /// Generates a new random 32-bit `Entity` id.
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
            Err(id::Error(s.to_string()))
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
    //! Entity id generation

    use rand::RngCore;

    /// Pulls in random data to generate a new 32 character hex string
    pub fn generate() -> String {
        let mut bytes: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Error converting from input string to Entity id
    #[derive(thiserror::Error, Debug)]
    pub struct Error(
        /// Input string
        pub String,
    );

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Invalid format for Entity Id: {}", self.0)
        }
    }
}

pub mod repository;
pub use repository::Repository;
pub mod service;
