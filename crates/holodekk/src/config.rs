use std::path::PathBuf;

use crate::repositories::RepositoryKind;
use crate::utils::ConnectionInfo;

pub trait HolodekkConfig: Clone + Sync + Send + 'static {
    fn data_root(&self) -> &PathBuf;
    fn exec_root(&self) -> &PathBuf;
    fn projectors_root(&self) -> &PathBuf;
    fn subroutines_root(&self) -> &PathBuf;
    fn bin_root(&self) -> &PathBuf;
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
    use std::path::Path;

    use rstest::*;

    use crate::utils::ConnectionInfo;

    use super::*;

    #[derive(Clone, Debug)]
    pub struct MockConfig {
        data_root: PathBuf,
        exec_root: PathBuf,
        projectors_root: PathBuf,
        subroutines_root: PathBuf,
        bin_root: PathBuf,
        projector_path: PathBuf,
        holodekk_api_config: ConnectionInfo,
    }

    impl MockConfig {
        pub fn new<P: AsRef<Path>>(data_root: P, exec_root: P) -> Self {
            let mut projectors_root = exec_root.as_ref().to_owned();
            projectors_root.push("projectors");
            let mut subroutines_root = exec_root.as_ref().to_owned();
            subroutines_root.push("subroutines");
            let mut holodekk_api_socket = exec_root.as_ref().to_owned();
            holodekk_api_socket.push("holodekkd.sock");

            let mut projector_path = exec_root.as_ref().to_owned();
            projector_path.push("projectors");
            projector_path.push("test");

            Self {
                data_root: data_root.as_ref().to_owned(),
                exec_root: exec_root.as_ref().to_owned(),
                projectors_root,
                subroutines_root,
                projector_path,
                bin_root: "/usr/local/bin".into(),
                holodekk_api_config: ConnectionInfo::unix(holodekk_api_socket),
            }
        }
    }

    impl Default for MockConfig {
        fn default() -> Self {
            let root: PathBuf = "/tmp".into();
            let mut data_root = root.clone();
            let mut exec_root = root.clone();
            data_root.push("data");
            exec_root.push("exec");
            Self::new(data_root, exec_root)
        }
    }

    impl HolodekkConfig for MockConfig {
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
