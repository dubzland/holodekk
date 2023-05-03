use std::path::Path;

use holodekk::{entity::repository, utils::ConnectionInfo, Paths};

#[derive(Clone, Debug)]
pub struct Config {
    paths: Paths,
    holodekk_api_config: ConnectionInfo,
    repo_kind: repository::Kind,
}

impl Config {
    pub fn new<P>(
        data_root: &P,
        exec_root: &P,
        bin_root: &P,
        holodekk_api_config: ConnectionInfo,
        repo_kind: repository::Kind,
    ) -> Self
    where
        P: AsRef<Path>,
    {
        let paths = Paths::new(data_root.as_ref(), exec_root.as_ref(), bin_root.as_ref());
        let mut holodekk_api_socket = exec_root.as_ref().to_owned();
        holodekk_api_socket.push("holodekkd.sock");

        Self {
            paths,
            holodekk_api_config,
            repo_kind,
        }
    }

    #[must_use]
    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    #[must_use]
    pub fn repo_kind(&self) -> repository::Kind {
        self.repo_kind
    }

    #[must_use]
    pub fn holodekk_api_config(&self) -> &ConnectionInfo {
        &self.holodekk_api_config
    }
}
