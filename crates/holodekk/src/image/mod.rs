//! Store-backed storage item.

use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref IMAGE_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

/// Newtype unique identifier for an `Image`
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(String);

impl Id {
    /// Generates a new 32-bit `Image` id based on a hash of the name.
    #[must_use]
    pub fn generate(name: &Name) -> Self {
        Self(id::generate(name))
    }
}

impl FromStr for Id {
    type Err = id::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if IMAGE_ID_RE.is_match(s) {
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

/// Image id generation
pub mod id {
    use sha2::{Digest, Sha256};

    /// creates a SHA-256 hash of the name to generate a 32-char hex id
    #[must_use]
    pub fn generate(name: &super::Name) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name);
        format!("{:x}", hasher.finalize())
    }

    /// Error converting from input string to Image id
    #[derive(thiserror::Error, Debug)]
    pub struct Error(
        /// Input string
        pub String,
    );

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Invalid format for Image Id: {}", self.0)
        }
    }
}

/// Newtype String wrapper to allow constraints to be placed on image name
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Name(String);

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&Name> for String {
    fn from(value: &Name) -> Self {
        value.0.clone()
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Name
where
    T: ?Sized,
    <Name as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}
