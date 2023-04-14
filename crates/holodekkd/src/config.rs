use std::path::PathBuf;

use holodekk::config::{HolodekkApiConfig, HolodekkConfig, HolodekkPaths};
use holodekk::core::repositories::RepositoryKind;
use holodekk::utils::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct HolodekkdConfig {
    fleet: String,
    paths: HolodekkPaths,
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
            paths: HolodekkPaths::new(root_path.into(), bin_path.into()),
            holodekk_api_config,
            repo_kind,
        }
    }
}

impl HolodekkConfig for HolodekkdConfig {
    fn fleet(&self) -> &str {
        &self.fleet
    }

    fn paths(&self) -> &HolodekkPaths {
        &self.paths
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
