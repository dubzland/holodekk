use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// Newtype wrapped around a `String` to allow constraints to be applied later
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Name(String);

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        name.0
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
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
