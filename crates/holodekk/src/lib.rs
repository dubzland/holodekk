//! Holodekk

#![warn(missing_docs)]

use std::path::{Path, PathBuf};

/// Required Holodekk path information.
///
/// These paths should be set during server initialization, and remain constant.
#[derive(Clone, Debug)]
pub struct Paths {
    data_root: PathBuf,
    exec_root: PathBuf,
    scenes_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
}

impl Paths {
    /// Creates a new set of Holodekk paths.
    ///
    /// Following convention over configuration, only the two top-level directories (`data_root`
    /// and `exec_root`) are configurable.  All child directories are derived from there.
    #[must_use]
    pub fn new<P>(data_root: P, exec_root: P, bin_root: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut scenes_root = exec_root.as_ref().to_owned();
        scenes_root.push("scenes");
        let mut subroutines_root = exec_root.as_ref().to_owned();
        subroutines_root.push("subroutines");

        Self {
            data_root: data_root.as_ref().to_owned(),
            exec_root: exec_root.as_ref().to_owned(),
            scenes_root,
            subroutines_root,
            bin_root: bin_root.as_ref().into(),
        }
    }

    /// Root directory for storing persistent data (images, configuration, etc)
    #[must_use]
    pub fn data_root(&self) -> &PathBuf {
        &self.data_root
    }

    /// Root directory for storing runtime data (sockets, pipes, etc)
    #[must_use]
    pub fn exec_root(&self) -> &PathBuf {
        &self.exec_root
    }

    /// Root directory within `exec_root` for storing runtime scene information
    #[must_use]
    pub fn scenes_root(&self) -> &PathBuf {
        &self.scenes_root
    }

    /// Root directory with in the `exec_root` for storing runtime subroutine information
    #[must_use]
    pub fn subroutines_root(&self) -> &PathBuf {
        &self.subroutines_root
    }

    /// Directory containing Holodekk executables
    #[must_use]
    pub fn bin_root(&self) -> &PathBuf {
        &self.bin_root
    }
}

pub mod apis;
pub mod entity;
pub mod errors;
pub mod image;
pub mod process;
pub mod scene;
pub mod subroutine;
pub mod utils;
