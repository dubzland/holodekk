use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::engine::Image;

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

#[derive(Debug)]
pub struct Subroutine {
    name: String,
    root_path: PathBuf,
    variant: String,

    pid_file: PathBuf,
    log_file: PathBuf,
    log_socket: PathBuf,
    shim_pid_file: PathBuf,
}

impl Subroutine {
    pub fn new(name: &str, root_path: &PathBuf, variant: &str) -> Self {
        let mut working_directory = root_path.clone();
        working_directory.push(".holodekk");
        let mut pid_file = working_directory.clone();
        pid_file.push("subroutine.pid");
        let mut log_file = working_directory.clone();
        log_file.push(format!("{}.log", variant));
        let mut log_socket = working_directory.clone();
        log_socket.push(format!("{}.sock", variant));
        let mut shim_pid_file = working_directory.clone();
        shim_pid_file.push(format!("{}.shim.pid", variant));

        Self {
            name: name.to_string(),
            root_path: root_path.to_owned(),
            variant: variant.to_string(),
            pid_file,
            log_file,
            log_socket,
            shim_pid_file,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
