use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref IMAGE_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(String);

impl Id {
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
    use sha2::{Digest, Sha256};

    pub fn generate(name: &super::Name) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name);
        format!("{:x}", hasher.finalize())
    }

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("Invalid ImageId format")]
        Format(String),
    }
}

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
