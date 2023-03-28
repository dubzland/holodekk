use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::engine::{Image, ImageKind};

#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    context: String,
    dockerfile: String,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.context, self.dockerfile)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Application {
    name: String,
    image: Option<Box<Image>>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            image: None,
        }
    }
}

impl Application {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_image(self, image: Box<Image>) -> Self {
        Self {
            image: Some(image),
            ..self
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subroutine {
    name: String,
    application: Application,
}

impl Subroutine {
    pub fn new(name: &str, application: Application) -> Self {
        Self {
            name: name.to_string(),
            application,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn application(&self) -> &Application {
        &self.application
    }

    pub fn from_manifest(manifest: &SubroutineManifest) -> Subroutine {
        let image = match manifest.container() {
            ContainerManifest::FromDockerContext {
                context: _,
                dockerfile: _,
            } => Image::new(manifest.name(), ImageKind::Application),
        };

        let application = Application::new(manifest.name()).with_image(Box::new(image));

        Self::new(manifest.name(), application)
    }
}

impl fmt::Display for Subroutine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContainerManifest {
    FromDockerContext { context: String, dockerfile: String },
}

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
