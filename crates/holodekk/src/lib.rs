use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct HolodekkPaths {
    data_root: PathBuf,
    exec_root: PathBuf,
    scenes_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
}

impl HolodekkPaths {
    pub fn new<P>(data_root: P, exec_root: P, bin_root: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut scenes_root = exec_root.as_ref().to_owned();
        scenes_root.push("scenes");
        let mut subroutines_root = exec_root.as_ref().to_owned();
        subroutines_root.push("subroutines");
        let mut holodekk_api_socket = exec_root.as_ref().to_owned();
        holodekk_api_socket.push("holodekkd.sock");

        Self {
            data_root: data_root.as_ref().to_owned(),
            exec_root: exec_root.as_ref().to_owned(),
            scenes_root,
            subroutines_root,
            bin_root: bin_root.as_ref().into(),
        }
    }

    pub fn data_root(&self) -> &PathBuf {
        &self.data_root
    }

    pub fn exec_root(&self) -> &PathBuf {
        &self.exec_root
    }

    pub fn scenes_root(&self) -> &PathBuf {
        &self.scenes_root
    }

    pub fn subroutines_root(&self) -> &PathBuf {
        &self.subroutines_root
    }

    pub fn bin_root(&self) -> &PathBuf {
        &self.bin_root
    }
}

pub mod apis;
pub mod core;
pub mod entities;
pub mod enums;
pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
// pub mod stores;
pub mod utils;
