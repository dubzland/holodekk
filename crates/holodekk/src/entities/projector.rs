use std::path::PathBuf;

use nix::unistd::Pid;
use sha2::{Digest, Sha256};

use crate::utils::ConnectionInfo;

fn generate_id<S: AsRef<str>>(fleet: S, namespace: S) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fleet.as_ref());
    hasher.update(namespace.as_ref());
    format!("{:x}", hasher.finalize())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Projector {
    pub id: String,
    pub fleet: String,
    pub namespace: String,
    pub pidfile: PathBuf,
    pub uhura_address: ConnectionInfo,
    pub projector_address: ConnectionInfo,
    pub pid: Pid,
}

impl Projector {
    pub fn new<S, P>(
        fleet: S,
        namespace: S,
        pidfile: P,
        uhura_address: ConnectionInfo,
        projector_address: ConnectionInfo,
        pid: Pid,
    ) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: Into<PathBuf>,
    {
        let id = generate_id(fleet.as_ref(), namespace.as_ref());
        Self {
            id,
            fleet: fleet.into(),
            namespace: namespace.into(),
            pidfile: pidfile.into(),
            uhura_address,
            projector_address,
            pid,
        }
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub(crate) fn projector() -> Projector {
        Projector::new(
            "test",
            "test",
            "/tmp/pid",
            ConnectionInfo::unix("/tmp/uhura.sock"),
            ConnectionInfo::unix("/tmp/projector.sock"),
            Pid::from_raw(123),
        )
    }
}
