use std::path::{Path, PathBuf};

use holodekk::{repositories::RepositoryKind, utils::ConnectionInfo, HolodekkPaths};

#[derive(Clone, Debug)]
pub struct HolodekkdConfig {
    paths: HolodekkPaths,
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
        let paths = HolodekkPaths::new(data_root.as_ref(), exec_root.as_ref(), bin_root.as_ref());
        let mut holodekk_api_socket = exec_root.as_ref().to_owned();
        holodekk_api_socket.push("holodekkd.sock");

        Self {
            paths,
            holodekk_api_config,
            repo_kind,
        }
    }

    pub fn paths(&self) -> &HolodekkPaths {
        &self.paths
    }

    pub fn repo_kind(&self) -> RepositoryKind {
        self.repo_kind
    }

    pub fn holodekk_api_config(&self) -> &ConnectionInfo {
        &self.holodekk_api_config
    }
}
