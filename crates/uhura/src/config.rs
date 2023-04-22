use std::path::{Path, PathBuf};

use holodekk::config::{HolodekkConfig, ProjectorApiConfig, ProjectorConfig, UhuraApiConfig};
use holodekk::repositories::RepositoryKind;
use holodekk::utils::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct UhuraConfig {
    namespace: String,
    data_root: PathBuf,
    exec_root: PathBuf,
    projectors_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
    projector_path: PathBuf,
    repo_kind: RepositoryKind,
    uhura_api_config: ConnectionInfo,
    projector_api_config: ConnectionInfo,
    pidfile: PathBuf,
}

impl UhuraConfig {
    pub fn new<P>(
        namespace: &str,
        data_root: P,
        exec_root: P,
        bin_root: P,
        repo_kind: RepositoryKind,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let mut projectors_root = exec_root.as_ref().to_owned();
        projectors_root.push("projectors");
        let mut subroutines_root = exec_root.as_ref().to_owned();
        subroutines_root.push("subroutines");

        let mut projector_path = exec_root.as_ref().to_owned();
        projector_path.push(namespace);

        let mut uhura_api_socket = projector_path.clone();
        uhura_api_socket.push("uhura.sock");
        let mut projector_api_socket = projector_path.clone();
        projector_api_socket.push("projector.sock");

        let mut pidfile = projector_path.clone();
        pidfile.push("uhura.pid");

        Self {
            namespace: namespace.into(),
            data_root: data_root.into(),
            exec_root: exec_root.into(),
            projectors_root,
            subroutines_root,
            bin_root: bin_root.into(),
            projector_path,
            repo_kind,
            uhura_api_config: ConnectionInfo::unix(uhura_api_socket),
            projector_api_config: ConnectionInfo::unix(projector_api_socket),
            pidfile,
        }
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }
}

impl HolodekkConfig for UhuraConfig {
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

impl ProjectorConfig for UhuraConfig {
    fn projector_path(&self) -> &PathBuf {
        &self.projector_path
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }
}

impl UhuraApiConfig for UhuraConfig {
    fn uhura_api_config(&self) -> &ConnectionInfo {
        &self.uhura_api_config
    }
}

impl ProjectorApiConfig for UhuraConfig {
    fn projector_api_config(&self) -> &ConnectionInfo {
        &self.projector_api_config
    }
}
