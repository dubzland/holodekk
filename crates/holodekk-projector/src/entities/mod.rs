use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct Subroutine {
    pub name: String,
    pub path: PathBuf,
}

impl Subroutine {
    pub fn new<S: AsRef<String>, P: AsRef<Path>>(name: S, path: P) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            path: path.as_ref().to_owned(),
        }
    }
}
