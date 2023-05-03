use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Paths as HolodekkPaths;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Kind {
    Unknown,
    Ruby,
}

impl Kind {
    pub fn detect<P: AsRef<Path>>(path: P) -> Kind {
        let mut ruby_path = PathBuf::from(path.as_ref());
        ruby_path.push("holodekk.rb");
        if ruby_path.try_exists().unwrap() {
            Self::Ruby
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug)]
pub struct Paths {
    root: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    socket: PathBuf,
}

impl Paths {
    pub fn build(paths: &HolodekkPaths, subroutine: &Entity) -> Self {
        let mut root = paths.subroutines_root().clone();
        root.push(subroutine.id.clone());

        let mut pidfile = root.clone();
        pidfile.push("subroutine.pid");

        let mut logfile = root.clone();
        logfile.push("subroutine.log");

        let mut socket = root.clone();
        socket.push("log.sock");

        Self {
            root,
            pidfile,
            logfile,
            socket,
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn logfile(&self) -> &PathBuf {
        &self.logfile
    }

    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

pub mod entity;
pub use entity::Entity;
