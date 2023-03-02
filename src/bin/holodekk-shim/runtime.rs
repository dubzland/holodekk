use std::ffi::{CString};
use std::io::Read;
use std::path::{Path, PathBuf};

use log::debug;

use holodekk::libsee;
use holodekk::streams::{create_pipes, override_streams};

pub trait Command {
    fn to_argv(&self, runtime: &str, id: &str) -> Vec<CString>;
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
    fn to_argv(&self, runtime: &str, id: &str) -> Vec<CString> {
        vec![
            CString::new(runtime).unwrap(),
            CString::new("create").unwrap(),
            CString::new("-b").unwrap(),
            CString::new(self.bundle_path.display().to_string()).unwrap(),
            CString::new(id).unwrap()
        ]
    }
}

pub struct ExecCommand {
    pidfile: PathBuf,
    arguments: Vec<String>
}

impl Command for ExecCommand {
    fn to_argv(&self, runtime: &str, id: &str) -> Vec<CString> {
        let mut argv = vec![
            CString::new(runtime).unwrap(),
            CString::new("exec").unwrap(),
            CString::new("--detach").unwrap(),
            CString::new("--pid-file").unwrap(),
            CString::new(self.pidfile.display().to_string()).unwrap(),
            CString::new(id).unwrap()
        ];

        for arg in self.arguments.iter() {
            argv.push(CString::new(arg.trim_matches('\'')).unwrap());
        }
        argv
    }
}

impl ExecCommand {
    pub fn new<F: AsRef<Path>>(pidfile: F, args: &Vec<String>) -> Self {
        Self {
            pidfile: pidfile.as_ref().to_owned(),
            arguments: args.to_owned()
        }
    }
}

pub struct Shim {
    runtime_path: PathBuf,
    container_id: String,
}

impl Shim {
    pub fn new<F: AsRef<Path>>(runtime_path: F) -> Self {
        Shim {
            runtime_path: runtime_path.as_ref().to_owned(),
            container_id: "".to_string(),
        }
    }

    pub fn container_id(mut self, id: &str) -> Self {
        self.container_id = id.to_owned();
        self
    }

    pub fn exec(&self, cmd: Box<dyn Command>) -> Option<libsee::Pid> {
        let res = libsee::fork().unwrap();
        if let Some(pid) = res {
            return Some(pid);
        }

        override_streams((None, None, None)).unwrap();
        libsee::setsid().unwrap();
        libsee::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0).unwrap();

        let (iomain, ioworker) = create_pipes(false, true, true);

        let res = libsee::fork().expect("unable to fork runtime process.");
        if let Some(pid) = res {
            drop(ioworker);
            let mut status: libc::c_int = -1;
            libsee::waitpid(pid, &mut status, 0).unwrap();

            if !libc::WIFEXITED(status) || libc::WEXITSTATUS(status) != 0 {
                debug!("worker failed: {}", libc::WEXITSTATUS(status));
            } else {
                debug!("worker exited with status: {}", libc::WEXITSTATUS(status));
            }
            let mut buf = Vec::new();
            let line = &mut buf;
            if let (_, _, Some(mut stderr)) = iomain.streams() {
                if let Err(err) = stderr.read_to_end(line) {
                    debug!("[shim] failed to read runtime's STDERR: {}", err);
                }
            }
            debug!("{}", String::from_utf8(line.to_vec()).unwrap_or(format!("{:?}", buf)));
        } else {
            drop(iomain);
            override_streams(ioworker.streams()).expect("Unable to override streams");
            let argv = cmd.to_argv(self.runtime_path.to_str().unwrap(), &self.container_id);
            libsee::execv(&argv);
        }
        None
    }

}
