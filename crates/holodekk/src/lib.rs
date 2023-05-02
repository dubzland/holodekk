use std::path::{Path, PathBuf};
use std::sync::Arc;

use entities::{SceneName, SubroutineEntity};

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

#[derive(Clone, Debug)]
pub struct ScenePaths {
    root: PathBuf,
    pidfile: PathBuf,
    socket: PathBuf,
}

impl ScenePaths {
    pub fn build(paths: &HolodekkPaths, name: &SceneName) -> Self {
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

#[derive(Debug)]
pub struct SubroutinePaths {
    root: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    socket: PathBuf,
}

impl SubroutinePaths {
    pub fn build(paths: Arc<HolodekkPaths>, subroutine: &SubroutineEntity) -> Self {
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

pub mod apis;
pub mod entities;
pub mod enums;
pub mod errors;
pub mod images;
pub mod models;
pub mod repositories;
pub mod services;
// pub mod stores;
pub mod utils;
