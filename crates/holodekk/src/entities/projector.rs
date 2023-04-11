use std::path::PathBuf;

use nix::unistd::Pid;
use uuid::Uuid;

use crate::utils::ConnectionInfo;

#[derive(Clone, Debug, PartialEq)]
pub struct Projector {
    pub id: Uuid,
    pub fleet: String,
    pub namespace: String,
    pub pidfile: PathBuf,
    pub uhura_address: ConnectionInfo,
    pub projector_address: ConnectionInfo,
    pub pid: Pid,
}
