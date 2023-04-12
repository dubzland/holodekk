use std::path::PathBuf;

use holodekk::config::{HolodekkConfig, ProjectorApiConfig, ProjectorConfig, UhuraApiConfig};
use holodekk::core::repositories::RepositoryKind;
use holodekk::utils::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct UhuraConfig {
    fleet: String,
    namespace: String,
    root_path: PathBuf,
    bin_path: PathBuf,
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
        P: Into<PathBuf>,
    {
        Self {
            fleet: fleet.into(),
            namespace: namespace.into(),
            root_path: root_path.into(),
            bin_path: bin_path.into(),
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

impl ProjectorConfig for UhuraConfig {
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
