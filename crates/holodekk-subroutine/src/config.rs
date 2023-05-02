use std::path::{Path, PathBuf};

use holodekk::repositories::RepositoryKind;
use holodekk_common::config::HolodekkConfig;

#[derive(Clone, Debug)]
pub struct SubroutineConfig {
    _path: PathBuf,
    data_root: PathBuf,
    exec_root: PathBuf,
    projectors_root: PathBuf,
    subroutines_root: PathBuf,
    bin_root: PathBuf,
    repo_kind: RepositoryKind,
    // projector_socket: PathBuf,
    _subroutine_id: String,
    shim_pidfile: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    log_socket: PathBuf,
}

impl SubroutineConfig {
    pub fn new<P, S>(
        path: P,
        data_root: P,
        exec_root: P,
        bin_root: P,
        repo_kind: RepositoryKind,
        subroutine_id: S,
        // projector_socket: P,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
        S: Into<String>,
    {
        let path: PathBuf = path.into();

        let mut projectors_root = exec_root.as_ref().to_owned();
        projectors_root.push("projectors");
        let mut subroutines_root = exec_root.as_ref().to_owned();
        subroutines_root.push("subroutines");

        let subroutine_id: String = subroutine_id.into();

        let mut root = subroutines_root.clone();
        root.push(subroutine_id.clone());

        let mut shim_pidfile = root.clone();
        shim_pidfile.push("shim.pid");

        let mut pidfile = root.clone();
        pidfile.push("subroutine.pid");

        let mut logfile = root.clone();
        logfile.push("subroutine.log");

        let mut log_socket = root;
        log_socket.push("log.sock");

        Self {
            _path: path,
            data_root: data_root.into(),
            exec_root: exec_root.into(),
            projectors_root,
            subroutines_root,
            bin_root: bin_root.into(),
            repo_kind,
            // projector_socket: projector_socket.into(),
            _subroutine_id: subroutine_id,
            shim_pidfile,
            pidfile,
            logfile,
            log_socket,
        }
    }

    // pub fn projector_socket(&self) -> &PathBuf {
    //     &self.projector_socket
    // }

    // pub fn path(&self) -> &PathBuf {
    //     &self.path
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

    // pub fn subroutine_id(&self) -> &str {
    //     &self.subroutine_id
    // }
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
