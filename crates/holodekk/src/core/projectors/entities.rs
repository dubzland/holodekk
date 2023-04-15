use std::path::PathBuf;

use nix::unistd::Pid;
use serde::{Serialize, Serializer};

use crate::utils::ConnectionInfo;

use super::repositories::projector_repo_id;

fn pid_serialize<S>(pid: &Pid, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i32(pid.as_raw())
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Projector {
    pub id: String,
    pub fleet: String,
    pub namespace: String,
    pub pidfile: PathBuf,
    pub uhura_address: ConnectionInfo,
    pub projector_address: ConnectionInfo,
    #[serde(serialize_with = "pid_serialize")]
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
        let id = projector_repo_id(fleet.as_ref(), namespace.as_ref());
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
