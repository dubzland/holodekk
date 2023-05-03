use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Paths {
    data_root: PathBuf,
    exec_root: PathBuf,
    scenes_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
}

impl Paths {
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
pub mod entity;
pub mod errors;
pub mod image;
pub mod scene;
pub mod subroutine;
pub mod utils;
