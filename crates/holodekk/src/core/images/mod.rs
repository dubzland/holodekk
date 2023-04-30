mod subroutine;
pub use subroutine::*;

use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ImageName(String);

impl std::fmt::Display for ImageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&ImageName> for String {
    fn from(value: &ImageName) -> Self {
        value.0.clone()
    }
}

impl From<&str> for ImageName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ImageName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Deref for ImageName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for ImageName
where
    T: ?Sized,
    <ImageName as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

pub fn generate_id(name: &ImageName) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name);
    format!("{:x}", hasher.finalize())
}

lazy_static! {
    static ref IMAGE_ID_RE: Regex = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum ImageIdError {
    #[error("Invalid ImageId format")]
    Format(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ImageId(String);

impl ImageId {
    pub fn generate(name: &ImageName) -> Self {
        Self(generate_id(name))
    }
}

impl FromStr for ImageId {
    type Err = ImageIdError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if IMAGE_ID_RE.is_match(s) {
            Ok(ImageId(s.to_string()))
        } else {
            Err(ImageIdError::Format(s.to_string()))
        }
    }
}

impl TryFrom<String> for ImageId {
    type Error = ImageIdError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl std::fmt::Display for ImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for ImageId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for ImageId
where
    T: ?Sized,
    <ImageId as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::core::enums::SubroutineKind;

    use super::*;

    #[fixture]
    pub fn mock_subroutine_image() -> SubroutineImage {
        SubroutineImage::new(
            "test/sub".into(),
            "/tmp/holodekk/subroutines/test/sub",
            SubroutineKind::Ruby,
        )
    }
}
