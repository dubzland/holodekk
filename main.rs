extern crate libc;

use std::path::PathBuf;
use std::ffi::{c_char, CString};

use libc::execv;

use clap::Parser;

use holodekk_shim::errors::{check_err, Errno};
use holodekk_shim::streams::{create_pipes, override_streams};

type Pid = libc::c_int;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    base: PathBuf,

    #[arg(short, long)]
    exec: bool,

    name: String,
}

fn main() {
    let args = Args::parse();

    println!("about to fork");

    unsafe {
        match fork_process() {
            Ok(Some(child_pid)) => {
                println!("child {} spawned.", child_pid);
                libc::_exit(0);
            },
            Err(err) => panic!("failed to fork child: {}", err),
            Ok(None) => (),
        }

        override_streams((None, None, None)).expect("override_streams failed");
        set_sid().expect("sessid() failed");
        make_subreaper().expect("make_subreaper failed");
    }

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
                if args.exec {
                    argv.push(CString::new("exec").unwrap());
                } else {
                    let bundle = format!("{}/bundle", args.base.display());
                    argv.push(CString::new("create").unwrap());
                    argv.push(CString::new("-b").unwrap());
                    argv.push(CString::new(bundle).unwrap());
                }

                argv.push(CString::new(args.name).unwrap());

                let mut args_raw: Vec<*const c_char> = argv.iter().map(|arg| arg.as_ptr()).collect();
                args_raw.push(std::ptr::null());

                execv(args_raw[0], args_raw.as_ptr());
            },
        }
    }
}

unsafe fn fork_process() -> Result<Option<Pid>, Errno> {
    let pid = check_err(libc::fork())?;
    if pid == 0 {
        Ok(None)
    } else {
        Ok(Some(pid))
    }
}

unsafe fn set_sid() -> Result<(), Errno> {
    check_err(libc::setsid())?;
    Ok(())
}

unsafe fn make_subreaper() -> Result<(), Errno> {
    check_err(libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0))?;
    Ok(())
}
