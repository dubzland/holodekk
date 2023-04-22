use std::path::{Path, PathBuf};

use holodekk::config::HolodekkConfig;
use holodekk::repositories::RepositoryKind;

#[derive(Clone, Debug)]
pub struct SubroutineConfig {
    data_root: PathBuf,
    exec_root: PathBuf,
    projectors_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
    repo_kind: RepositoryKind,
    // projector_socket: PathBuf,
    shim_pidfile: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    log_socket: PathBuf,
}

impl SubroutineConfig {
    pub fn new<P>(
        path: P,
        data_root: P,
        exec_root: P,
        bin_root: P,
        repo_kind: RepositoryKind,
        // projector_socket: P,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        let mut projectors_root = exec_root.as_ref().to_owned();
        projectors_root.push("projectors");
        let mut subroutines_root = exec_root.as_ref().to_owned();
        subroutines_root.push("subroutines");

        let mut shim_pidfile = path.clone();
        shim_pidfile.push("shim.pid");

        let mut pidfile = path.clone();
        pidfile.push("subroutine.pid");

        let mut logfile = path.clone();
        logfile.push("subroutine.log");

        let mut log_socket = path;
        log_socket.push("log.sock");

        Self {
            data_root: data_root.into(),
            exec_root: exec_root.into(),
            projectors_root,
            subroutines_root,
            bin_root: bin_root.into(),
            repo_kind,
            // projector_socket: projector_socket.into(),
            shim_pidfile,
            pidfile,
            logfile,
            log_socket,
        }
    }

    // pub fn projector_socket(&self) -> &PathBuf {
    //     &self.projector_socket
    // }

    pub fn shim_pidfile(&self) -> &PathBuf {
        &self.shim_pidfile
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn logfile(&self) -> &PathBuf {
        &self.logfile
    }

    pub fn log_socket(&self) -> &PathBuf {
        &self.log_socket
    }
}

impl HolodekkConfig for SubroutineConfig {
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
