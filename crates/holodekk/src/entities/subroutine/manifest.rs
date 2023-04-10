use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entities::ContainerManifest;

/// Object derived from dumping the subroutine configured by an extension.
///
/// This should be considered a read-only view of the subroutine.  It exists merely
/// to allow the subroutine to be identified prior to actual execution.
///
/// The entity actually used by the system for managing instances is `Subroutine`.
///
/// # Examples
///
/// ```rust,ignore
/// use holodekk_core::holodekk::SubroutineManifest;
/// // load the json for a subroutine
/// let manifest: SubroutineManifest = serde_json::from_str(&json).unwrap();
#[derive(Debug, Deserialize, Serialize)]
pub struct SubroutineManifest {
    // fleet: String,
    // namespace: String,
    name: String,
    container: ContainerManifest,
    environment: Option<HashMap<String, String>>,
    port: Option<i16>,
}

impl SubroutineManifest {
    /// Name as set by the end user.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Container manifest included in the extension.
    pub fn container(&self) -> &ContainerManifest {
        &self.container
    }

    /// Environment variables to be set on subroutine execution.
    pub fn environment(&self) -> Option<&HashMap<String, String>> {
        self.environment.as_ref()
    }

    /// Port number the specified container will expect traffic when executed.
    pub fn port(&self) -> Option<&i16> {
        self.port.as_ref()
    }
}

impl fmt::Display for SubroutineManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}", self.name)
    }
}
