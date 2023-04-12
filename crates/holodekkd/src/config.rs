use std::path::PathBuf;

use holodekk::config::{HolodekkApiConfig, HolodekkConfig};
use holodekk::core::repositories::RepositoryKind;
use holodekk::utils::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct HolodekkdConfig {
    fleet: String,
    root_path: PathBuf,
    bin_path: PathBuf,
    holodekk_api_config: ConnectionInfo,
    repo_kind: RepositoryKind,
}

impl HolodekkdConfig {
    pub fn new<P>(
        fleet: &str,
        root_path: P,
        bin_path: P,
        holodekk_api_config: ConnectionInfo,
        repo_kind: RepositoryKind,
    ) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            fleet: fleet.into(),
            root_path: root_path.into(),
            bin_path: bin_path.into(),
            holodekk_api_config,
            repo_kind,
        }
    }
}

impl HolodekkConfig for HolodekkdConfig {
    fn fleet(&self) -> &str {
        &self.fleet
    }

    fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    fn bin_path(&self) -> &PathBuf {
        &self.bin_path
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
