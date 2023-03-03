use std::ffi::{CString};
use std::io::Read;
use std::path::{Path, PathBuf};

use log::debug;

use libsee::Pid;

use holodekk::libsee;
use holodekk::streams::{create_pipes, override_streams};

pub trait Command {
    fn to_argv(&self, runtime: &str, pidfile: &str, id: &str) -> Vec<CString>;
}

pub struct CreateCommand {
    bundle_path: PathBuf
}

impl CreateCommand {
    pub fn new<F: AsRef<Path>>(bundle_path: F) -> Self {
        Self {
            bundle_path: bundle_path.as_ref().to_owned()
        }
    }
}

impl Command for CreateCommand {
    fn to_argv(&self, runtime: &str, pidfile: &str, id: &str) -> Vec<CString> {
        vec![
            CString::new(runtime).unwrap(),
            CString::new("create").unwrap(),
            CString::new("--pid-file").unwrap(),
            CString::new(pidfile).unwrap(),
            CString::new("--bundle").unwrap(),
            CString::new(self.bundle_path.display().to_string()).unwrap(),
            CString::new(id).unwrap()
        ]
    }
}

pub struct ExecCommand {
    arguments: Vec<String>
}

impl ExecCommand {
    pub fn new(args: &Vec<String>) -> Self {
        Self {
            arguments: args.to_owned()
        }
    }
}

impl Command for ExecCommand {
    fn to_argv(&self, runtime: &str, pidfile: &str, id: &str) -> Vec<CString> {
        let mut argv = vec![
            CString::new(runtime).unwrap(),
            CString::new("exec").unwrap(),
            CString::new("--detach").unwrap(),
            CString::new("--pid-file").unwrap(),
            CString::new(pidfile).unwrap(),
            CString::new(id).unwrap()
        ];

        for arg in self.arguments.iter() {
            argv.push(CString::new(arg.trim_matches('\'')).unwrap());
        }
        argv
    }
}

pub struct Container {
    id: String,
    pidfile: PathBuf,
    pid: Pid,
}

impl Container {
    pub fn new<F: AsRef<Path>>(id: &str, pidfile: F) -> Self {
        Container {
            id: id.to_string(),
            pidfile: pidfile.as_ref().to_owned(),
            pid: -1
        }
    }
}

pub fn exec<F: AsRef<Path>>(runtime_path: F, container: &mut Container, cmd: Box<dyn Command>) {
    override_streams((None, None, None)).unwrap();
    libsee::setsid().unwrap();
    libsee::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0).unwrap();

    let (iomain, ioworker) = create_pipes(false, true, true);

    let res = libsee::fork().expect("unable to fork runtime process.");
    if let Some(pid) = res {
        debug!("runtime forked with pid {}", pid);
        drop(ioworker);
        let mut status: libc::c_int = -1;
        libsee::waitpid(pid, &mut status, 0).unwrap();

        if !libc::WIFEXITED(status) || libc::WEXITSTATUS(status) != 0 {
            debug!("runtime failed: {}", libc::WEXITSTATUS(status));
            let mut buf = Vec::new();
            let line = &mut buf;
            if let (_, _, Some(mut stderr)) = iomain.streams() {
                if let Err(err) = stderr.read_to_end(line) {
                    debug!("failed to read runtime's STDERR: {}", err);
                }
            }
            debug!("{}", String::from_utf8(line.to_vec()).unwrap_or(format!("{:?}", buf)));
        } else {
            debug!("runtime exited with status: {}", libc::WEXITSTATUS(status));
        }
    } else {
        drop(iomain);
        override_streams(ioworker.streams()).expect("Unable to override streams");
        let argv = cmd.to_argv(
            runtime_path.as_ref().to_str().unwrap(),
            container.pidfile.to_str().unwrap(),
            &container.id
        );
        libsee::execv(&argv);
        libsee::_exit(127);
    }
}
