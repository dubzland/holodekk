use std::path::{Path, PathBuf};

use holodekk::config::{HolodekkApiConfig, HolodekkConfig};
use holodekk::repositories::RepositoryKind;
use holodekk::utils::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct HolodekkdConfig {
    data_root: PathBuf,
    exec_root: PathBuf,
    projectors_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
    holodekk_api_config: ConnectionInfo,
    repo_kind: RepositoryKind,
}

impl HolodekkdConfig {
    pub fn new<P>(
        data_root: P,
        exec_root: P,
        bin_root: P,
        holodekk_api_config: ConnectionInfo,
        repo_kind: RepositoryKind,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let mut projectors_root = exec_root.as_ref().to_owned();
        projectors_root.push("projectors");
        let mut subroutines_root = exec_root.as_ref().to_owned();
        subroutines_root.push("subroutines");
        let mut holodekk_api_socket = exec_root.as_ref().to_owned();
        holodekk_api_socket.push("holodekkd.sock");

        Self {
            data_root: data_root.into(),
            exec_root: exec_root.into(),
            projectors_root,
            subroutines_root,
            bin_root: bin_root.into(),
            holodekk_api_config,
            repo_kind,
        }
    }
}

impl HolodekkConfig for HolodekkdConfig {
    fn data_root(&self) -> &PathBuf {
        &self.data_root
    }

    fn exec_root(&self) -> &PathBuf {
        &self.exec_root
    }

    fn projectors_root(&self) -> &PathBuf {
        &self.projectors_root
    }

    fn subroutines_root(&self) -> &PathBuf {
        &self.subroutines_root
    }

    fn bin_root(&self) -> &PathBuf {
        &self.bin_root
    }

    fn repo_kind(&self) -> RepositoryKind {
        self.repo_kind
    }
}

impl HolodekkApiConfig for HolodekkdConfig {
    fn holodekk_api_config(&self) -> &ConnectionInfo {
        &self.holodekk_api_config
    }
}
