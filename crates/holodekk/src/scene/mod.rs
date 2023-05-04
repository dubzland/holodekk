//! Scenes

use std::path::PathBuf;

use super::Paths as HolodekkPaths;

/// Paths (on disk) where a given scene is running
#[derive(Clone, Debug)]
pub struct Paths {
    root: PathBuf,
    pidfile: PathBuf,
    socket: PathBuf,
}

impl Paths {
    /// Builds a set of paths based on the currently running Holodekk instance for this scene.
    #[must_use]
    pub fn build(paths: &HolodekkPaths, name: &entity::Name) -> Self {
        let mut root = paths.scenes_root().clone();
        root.push(name);

        let mut pidfile = root.clone();
        pidfile.push("uhura.pid");

        let mut socket = root.clone();
        socket.push("uhura.sock");

        Self {
            root,
            pidfile,
            socket,
        }
    }

    /// The root directory for this scene
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Location on disk of this scene projector's pidfile
    #[must_use]
    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    /// Location on disk of this scene projector's socket
    #[must_use]
    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

pub mod entity;
pub use entity::Entity;
pub mod monitor;
pub use monitor::Monitor;
