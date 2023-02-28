use std::ffi::{c_char, CString};
use std::fs;
use std::path::{Path, PathBuf};

use libc::execv;

use log::debug;

// use crate::errors::{check_err, Errno};
use crate::process::{fork_process, make_subreaper, Pid, set_sid};
use crate::streams::{create_pipes, override_streams};

#[derive(Eq, PartialEq)]
enum ShimMode {
    Create,
    Exec,
}

pub struct Shim {
    bundle: PathBuf,
    pid_file: Option<PathBuf>,
}

impl Shim {
    pub fn new<F: AsRef<Path>>(bundle: F) -> Self{
        Shim {
            bundle: bundle.as_ref().to_owned(),
            pid_file: None,
        }
    }

    pub fn pid_file<F: AsRef<Path>>(mut self, path: F) -> Self {
        self.pid_file = Some(path.as_ref().to_owned());
        self
    }

    pub fn exec(&self, name: String) {
        self.setup();
        self.exec_runtime(ShimMode::Exec, name);
    }

    pub fn create(&self, name: String) {
        self.setup();
        self.exec_runtime(ShimMode::Create, name);
    }

    pub fn setup(&self) {
        debug!("In shim::start()");
        debug!("bundle:   {}", self.bundle.display());
        debug!("pid_file: {:?}", self.pid_file);
        unsafe {
            let res = fork_process().expect("unable to fork shim process.");
            if let Some(pid) = res {
                self.write_pid(pid);
                // child forked
                libc::_exit(0);
            }
            override_streams((None, None, None)).expect("override_streams failed");
            set_sid().expect("sessid() failed");
            make_subreaper().expect("make_subreaper failed");
        }
    }

    fn write_pid(&self, pid: Pid) {
        if let Some(pidfile) = &self.pid_file {
            if let Err(err) = fs::write(&pidfile, format!("{}", pid)) {
                panic!("write() to pidfile {} failed: {}", pidfile.display(), err);
            }
        }
    }

    fn exec_runtime(&self, mode: ShimMode, name: String) {
        let (iomain, ioworker) = create_pipes(false, true, true);

        unsafe {
            match fork_process() {
                Ok(Some(child_pid)) => {
                    drop(ioworker);
                    let mut status: libc::c_int = -1;
                    libc::waitpid(child_pid, &mut status, 0);
                },
                Err(err) => panic!("failed to fork child: {}", err),
                Ok(None) => {
                    drop(iomain);
                    override_streams(ioworker.streams()).expect("Unable to override streams");
                    let mut argv: Vec<CString> = Vec::new();
                    argv.push(CString::new("/usr/bin/runc").unwrap());
                    if mode == ShimMode::Exec {
                        argv.push(CString::new("exec").unwrap());
                    } else {
                        let bundle = format!("{}/bundle", self.bundle.display());
                        argv.push(CString::new("create").unwrap());
                        argv.push(CString::new("-b").unwrap());
                        argv.push(CString::new(bundle).unwrap());
                    }

                    argv.push(CString::new(name).unwrap());

                    let mut args_raw: Vec<*const c_char> = argv.iter().map(|arg| arg.as_ptr()).collect();
                    args_raw.push(std::ptr::null());

                    execv(args_raw[0], args_raw.as_ptr());
                },
            }
        }
    }
}

