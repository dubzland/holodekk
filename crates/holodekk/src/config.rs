use std::path::{Path, PathBuf};

use crate::core::repositories::RepositoryKind;
use crate::utils::ConnectionInfo;

#[derive(Clone, Debug, PartialEq)]
pub struct HolodekkPaths {
    root: PathBuf,
    projectors: PathBuf,
    subroutines: PathBuf,
    bin: PathBuf,
}

impl HolodekkPaths {
    pub fn new<P: AsRef<Path>>(root: P, bin: P) -> Self {
        let root = root.as_ref().to_owned();
        let mut projectors = root.clone();
        projectors.push("projectors");
        let mut subroutines = root.clone();
        subroutines.push("subroutines");

        Self {
            root,
            projectors,
            subroutines,
            bin: bin.as_ref().to_owned(),
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn projectors(&self) -> &PathBuf {
        &self.projectors
    }

    pub fn subroutines(&self) -> &PathBuf {
        &self.subroutines
    }

    pub fn bin(&self) -> &PathBuf {
        &self.bin
    }
}

pub trait HolodekkConfig: Clone + Sync + Send + 'static {
    fn fleet(&self) -> &str;
    fn paths(&self) -> &HolodekkPaths;
    fn repo_kind(&self) -> RepositoryKind;
}

pub trait ProjectorConfig: Clone + Sync + Send + 'static {
    fn projector_path(&self) -> &PathBuf;
    fn namespace(&self) -> &str;
}

pub trait HolodekkApiConfig: Clone + Sync + Send + 'static {
    fn holodekk_api_config(&self) -> &ConnectionInfo;
}

pub trait ProjectorApiConfig: Clone + Sync + Send + 'static {
    fn projector_api_config(&self) -> &ConnectionInfo;
}

pub trait UhuraApiConfig: Clone + Sync + Send + 'static {
    fn uhura_api_config(&self) -> &ConnectionInfo;
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::utils::ConnectionInfo;

    use super::*;

    #[derive(Clone, Debug)]
    pub struct MockConfig {
        paths: HolodekkPaths,
        projector_path: PathBuf,
        holodekk_api_config: ConnectionInfo,
    }

    impl MockConfig {
        pub fn new<P: AsRef<Path>>(root: P) -> Self {
            let mut holodekk_api_socket = root.as_ref().to_owned();
            holodekk_api_socket.push("holodekkd.sock");

            let mut projector_path = root.as_ref().to_owned();
            projector_path.push("projectors");
            projector_path.push("test");

            Self {
                paths: HolodekkPaths::new(root.as_ref().to_owned(), PathBuf::from("/tmp/bin")),
                projector_path,
                holodekk_api_config: ConnectionInfo::unix(holodekk_api_socket),
            }
        }
    }

    impl Default for MockConfig {
        fn default() -> Self {
            let holodekk_root_path: PathBuf = "/tmp".into();
            Self::new(holodekk_root_path)
        }
    }

    impl HolodekkConfig for MockConfig {
        fn fleet(&self) -> &str {
            "test"
        }

        fn paths(&self) -> &HolodekkPaths {
            &self.paths
        }

        fn repo_kind(&self) -> RepositoryKind {
            RepositoryKind::Memory
        }
    }

    impl ProjectorConfig for MockConfig {
        fn projector_path(&self) -> &PathBuf {
            &self.projector_path
        }

        fn namespace(&self) -> &str {
            "test"
        }
    }

    impl HolodekkApiConfig for MockConfig {
        fn holodekk_api_config(&self) -> &ConnectionInfo {
            &self.holodekk_api_config
        }
    }

    #[fixture]
    pub fn mock_config() -> MockConfig {
        MockConfig::default()
    }
}
