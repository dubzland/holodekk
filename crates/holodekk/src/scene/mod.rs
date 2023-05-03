use std::path::PathBuf;

use super::Paths as HolodekkPaths;

#[derive(Clone, Debug)]
pub struct Paths {
    root: PathBuf,
    pidfile: PathBuf,
    socket: PathBuf,
}

impl Paths {
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

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

pub mod entity;
pub use entity::Entity;
pub mod monitor;
pub use monitor::Monitor;
