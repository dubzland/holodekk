use std::path::PathBuf;

use crate::core::repositories::RepositoryKind;
use crate::utils::ConnectionInfo;

pub trait HolodekkConfig: Clone + Sync + Send + 'static {
    fn fleet(&self) -> &str;
    fn root_path(&self) -> &PathBuf;
    fn bin_path(&self) -> &PathBuf;
    fn repo_kind(&self) -> RepositoryKind;
}

pub trait ProjectorConfig {
    fn namespace(&self) -> &str;
}

pub trait HolodekkApiConfig {
    fn holodekk_api_config(&self) -> &ConnectionInfo;
}

pub trait ProjectorApiConfig {
    fn projector_api_config(&self) -> &ConnectionInfo;
}

pub trait UhuraApiConfig {
    fn uhura_api_config(&self) -> &ConnectionInfo;
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[derive(Clone, Debug)]
    pub struct MockConfig {
        root_path: PathBuf,
        bin_path: PathBuf,
    }

    impl Default for MockConfig {
        fn default() -> Self {
            Self {
                root_path: "/tmp".into(),
                bin_path: "/tmp/bin".into(),
            }
        }
    }

    impl HolodekkConfig for MockConfig {
        fn fleet(&self) -> &str {
            "test"
        }

        fn root_path(&self) -> &PathBuf {
            &self.root_path
        }

        fn bin_path(&self) -> &PathBuf {
            &self.bin_path
        }

        fn repo_kind(&self) -> RepositoryKind {
            RepositoryKind::Memory
        }
    }

    impl ProjectorConfig for MockConfig {
        fn namespace(&self) -> &str {
            "test"
        }
    }

    #[fixture]
    pub fn mock_config() -> MockConfig {
        MockConfig::default()
    }
}
