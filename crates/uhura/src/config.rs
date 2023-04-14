use std::path::{Path, PathBuf};

use holodekk::config::{
    HolodekkConfig, HolodekkPaths, ProjectorApiConfig, ProjectorConfig, UhuraApiConfig,
};
use holodekk::core::repositories::RepositoryKind;
use holodekk::utils::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct UhuraConfig {
    fleet: String,
    namespace: String,
    paths: HolodekkPaths,
    projector_path: PathBuf,
    repo_kind: RepositoryKind,
    uhura_api_config: ConnectionInfo,
    projector_api_config: ConnectionInfo,
}

impl UhuraConfig {
    pub fn new<P>(
        fleet: &str,
        namespace: &str,
        root_path: P,
        bin_path: P,
        repo_kind: RepositoryKind,
        uhura_api_config: ConnectionInfo,
        projector_api_config: ConnectionInfo,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let paths = HolodekkPaths::new(root_path, bin_path);
        let mut projector_path = paths.root().clone();
        projector_path.push(namespace);

        Self {
            fleet: fleet.into(),
            namespace: namespace.into(),
            paths,
            projector_path,
            repo_kind,
            uhura_api_config,
            projector_api_config,
        }
    }
}

impl HolodekkConfig for UhuraConfig {
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

impl ProjectorConfig for UhuraConfig {
    fn projector_path(&self) -> &PathBuf {
        &self.projector_path
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }
}

impl ProjectorApiConfig for UhuraConfig {
    fn projector_api_config(&self) -> &ConnectionInfo {
        &self.projector_api_config
    }
}

impl UhuraApiConfig for UhuraConfig {
    fn uhura_api_config(&self) -> &ConnectionInfo {
        &self.uhura_api_config
    }
}
