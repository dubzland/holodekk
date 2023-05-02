use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SceneName(String);

impl std::fmt::Display for SceneName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SceneName> for String {
    fn from(name: SceneName) -> Self {
        name.0
    }
}

impl From<String> for SceneName {
    fn from(value: String) -> Self {
        SceneName(value)
    }
}

impl From<&str> for SceneName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Deref for SceneName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for SceneName {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl<T> AsRef<T> for SceneName
where
    T: ?Sized,
    <SceneName as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}
