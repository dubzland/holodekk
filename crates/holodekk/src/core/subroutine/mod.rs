//! Subroutines
//!
//! From an overall operations point of view, the `Subroutine` is the driving force behind
//! everything.  If you want to run something on the Holodekk, you must create a subroutine to
//! direct the engine to do your bidding.
//!
//! At its core, each Subroutine is just a small process running in the background, but it can both
//! make requests of and respond to events from the Holodekk.

use std::path::{Path, PathBuf};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::Paths as HolodekkPaths;

/// The kind of framework the subroutine is based on.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Kind {
    /// Unable to determine the subroutine kind
    Unknown,
    /// A Ruby(gems) based subroutine
    Ruby,
}

impl Kind {
    /// Attempts to detect the kind of subroutine a particular directory contains
    pub fn detect<P: AsRef<Path>>(path: P) -> Kind {
        let mut ruby_path = PathBuf::from(path.as_ref());
        ruby_path.push("holodekk.rb");
        match ruby_path.try_exists() {
            Err(err) => {
                warn!("Error encountered trying to detect subroutine type: {err}");
                Self::Unknown
            }
            Ok(exists) => {
                if exists {
                    Self::Ruby
                } else {
                    Self::Unknown
                }
            }
        }
    }
}

/// Paths (on disk) where a given subroutine is running
#[derive(Debug)]
pub struct Paths {
    root: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    socket: PathBuf,
}

impl Paths {
    /// Builds a set of paths based on the currently running Holodekk instance for this subroutine.
    #[must_use]
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

    /// The root directory for this subroutine
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Location on disk of this subroutine shim's pidfile
    #[must_use]
    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    /// Location on disk of this subroutine's logfile
    #[must_use]
    pub fn logfile(&self) -> &PathBuf {
        &self.logfile
    }

    /// Location on disk of this subroutine's log socket
    #[must_use]
    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

pub mod entity;
pub use entity::Entity;
pub mod image;
pub use image::Image;
