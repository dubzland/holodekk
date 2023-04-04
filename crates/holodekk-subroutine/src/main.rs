mod logger;
mod server;
mod signals;
mod streams;

use std::ffi::CString;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::panic;
use std::path::PathBuf;

use clap::Parser;

use log::{debug, error, warn, LevelFilter};

use nix::{
    fcntl::OFlag,
    sys::{
        signal::{
            kill, sigprocmask, SigSet, SigmaskHow, SIGCHLD, SIGINT, SIGKILL, SIGQUIT, SIGTERM,
        },
        wait::waitpid,
    },
    unistd::{close, dup2, execv, fork, pipe2, setsid, ForkResult, Pid},
};

use syslog::{BasicLogger, Facility, Formatter3164};

use serde::{Deserialize, Serialize};

use holodekk_utils::libsee;

use server::Server;
use signals::signal_mask;
use streams::open_dev_null;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// Name of the subroutine to execute
    #[arg(long, short, required = true)]
    name: String,

    /// Path to the shim's pid file
    #[arg(long = "shim-pidfile", value_name = "file", required = true)]
    pidfile: PathBuf,

    /// Working directory for the subroutine
    #[arg(long = "subroutine-directory", value_name = "dir", required = true)]
    subroutine_directory: PathBuf,

    /// Path to the container's pid file
    #[arg(long = "subroutine-pidfile", value_name = "file", required = true)]
    subroutine_pidfile: PathBuf,

    /// Path to the subroutine log file
    #[arg(long = "log-file", value_name = "file", required = true)]
    log_file: PathBuf,

    /// Variant to execute
    #[arg(long = "subroutine", value_name = "name", required = true)]
    subroutine: String,

    /// Projector port
    #[arg(short, long, required = true)]
    projector_port: String,

    /// Log socket
    #[arg(long, required = true)]
    log_socket: PathBuf,

    /// Sync pipe
    #[arg(long)]
    sync_pipe: i32,

    #[arg(long)]
    reconnect_log: bool,
}

#[derive(Debug, Serialize)]
struct MessageSubroutinePid {
    kind: &'static str,
    pid: i32,
}

#[derive(Debug, Deserialize)]
struct MessageSubroutinePidParent {
    kind: String,
    pid: i32,
}

impl MessageSubroutinePid {
    fn new(pid: Pid) -> Self {
        MessageSubroutinePid {
            kind: "subroutine_pid",
            pid: pid.as_raw(),
        }
    }
}

fn connect_log_stream(log_socket: &PathBuf) {
    let mut stream = UnixStream::connect(log_socket).unwrap();

    loop {
        let mut buf = [0; 1024];
        let bytes_read = stream.read(&mut buf).unwrap();
        if buf[0] == 1 {
            io::stdout().write_all(&buf[1..bytes_read]).unwrap();
        } else if buf[0] == 2 {
            io::stderr().write_all(&buf[1..bytes_read]).unwrap();
        }
    }
}

fn pretend_to_be_container_manager(sync_fd: i32, log_socket: &PathBuf) {
    let mut sync_pipe = unsafe { File::from_raw_fd(sync_fd) };
    let mut buf = [0; 1024];
    let bytes_read = sync_pipe
        .read(&mut buf)
        .expect("failed to read from the sync pipe");
    let msg: MessageSubroutinePidParent =
        serde_json::from_slice(&buf[0..bytes_read]).expect("failed to deserialize JSON");

    println!("Got update from shim:");
    println!("kind: {}", msg.kind);
    println!("pid:  {}", msg.pid);

    connect_log_stream(log_socket);
}

fn main() {
    let options = Options::parse();

    if options.reconnect_log {
        connect_log_stream(&options.log_socket);
        return;
    }
    // TEMP: Fake the holodekk sync pipe
    let (parent_fd, child_fd) = pipe2(OFlag::O_CLOEXEC).unwrap();

    // Perform the initial fork
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            write_master_pidfile(&options.pidfile, child);
            pretend_to_be_container_manager(parent_fd, &options.log_socket);

            libsee::_exit(0);
        }
        Ok(ForkResult::Child) => (),
        Err(_) => {
            eprintln!("Fork failed");
            libsee::_exit(1);
        }
    }

    init_logger(LevelFilter::Debug);

    // TEMP
    close(parent_fd).unwrap();

    // Redirect all streams to /dev/null
    let (dev_null_rd, dev_null_wr) = open_dev_null();
    dup2(dev_null_rd, libsee::STDIN_FILENO).expect("Failed to redirect stdin to /dev/null");
    dup2(dev_null_wr, libsee::STDOUT_FILENO).expect("Failed to redirect stdout to /dev/null");
    dup2(dev_null_wr, libsee::STDERR_FILENO).expect("Failed to redirect stderr to /dev/null");

    // new session
    setsid().expect("Failed to create new session");

    // make us a subreaper
    libsee::prctl(libsee::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0)
        .expect("Unable to set ourselves as subreaper");

    // block signals (until we're ready)
    let mut oldmask = SigSet::empty();
    let signals = signal_mask(&[SIGCHLD, SIGINT, SIGQUIT, SIGTERM]);
    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&signals), Some(&mut oldmask))
        .expect("failed to block signals");

    // create our io pipes
    let (main_stdout, worker_stdout) =
        pipe2(OFlag::O_CLOEXEC).expect("Failed to create stdout pipes");
    let (main_stderr, worker_stderr) =
        pipe2(OFlag::O_CLOEXEC).expect("Failed to create stderr pipes");

    // fork again
    let child_pid = match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => child,
        Ok(ForkResult::Child) => {
            // ensure we die if our parent disappears
            libsee::prctl(
                libsee::PR_SET_PDEATHSIG,
                SIGKILL as libsee::c_ulong,
                0,
                0,
                0,
            )
            .expect("failed to set death signal");

            // restore signals
            sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oldmask), None)
                .expect("failed to restore signals");

            // close the master pipes
            close(main_stdout).expect("Failed to close main stdout in worker process");
            close(main_stderr).expect("Failed to close main stderr in worker process");
            close(child_fd).unwrap();

            // capture io (ignoring stdin)
            dup2(dev_null_rd, libsee::STDIN_FILENO)
                .expect("Failed to redirect stdin to /dev/null in worker process");
            dup2(worker_stdout, libsee::STDOUT_FILENO)
                .expect("Failed to redirect stdout in worker process");
            dup2(worker_stderr, libsee::STDERR_FILENO)
                .expect("Failed to redirect stderr in worker process");

            // launch the subroutine
            // let argv = vec![
            //     CString::new(options.subroutine).unwrap(),
            //     // CString::new("--pid-file").unwrap(),
            //     // CString::new(pidfile).unwrap(),
            //     CString::new("--projector-port").unwrap(),
            //     CString::new(options.projector_port).unwrap(),
            // ];
            let argv = vec![
                CString::new("/usr/bin/ping").unwrap(),
                CString::new("127.0.0.1").unwrap(),
            ];
            execv(&argv[0], &argv).unwrap_or_else(|_| libsee::_exit(127));
            panic!("we should never get here");
        }
        Err(err) => {
            eprintln!("Fork failed");
            panic!("fork() of the subroutine process failed: {}", err);
        }
    };

    // close the worker pipes
    close(worker_stdout).expect("Failed to close worker stdout in main process");
    close(worker_stderr).expect("failed to close worker stderr in main process");

    // start the server to monitor the subroutine and serve logs
    let result = Server::build()
        .with_child(child_pid)
        .with_stdio(main_stdout, main_stderr)
        .with_log_file(&options.log_file)
        .listen_uds(&options.log_socket);

    match result {
        Ok(mut server) => {
            // Notify the parent of our state
            match serde_json::to_vec(&MessageSubroutinePid::new(child_pid)) {
                Ok(msg) => {
                    let mut sync_pipe = unsafe { File::from_raw_fd(child_fd) };
                    if sync_pipe.write_all(&msg).is_err() {
                        warn!("Failed to write status to sync pipe");
                    }
                }
                Err(err) => {
                    warn!("Failed to serialize JSON for sync update: {}", err);
                }
            }

            // Run the server
            match server.run() {
                Ok(status) => {
                    debug!("subroutine exited with status: {:?}", status);
                }
                Err(err) => {
                    // server terminated abnormally
                    // Make absolutely sure we've reaped the child
                    warn!("Server exited abnormally: {:?}", err);
                    ensure_child_reaped(child_pid);
                }
            }
        }
        Err(err) => {
            warn!("Error building server: {}", err);
            ensure_child_reaped(child_pid);
        }
    }
}

fn write_master_pidfile(pidfile: &PathBuf, pid: Pid) {
    debug!("forked worker with pid: {}", pid);
    if let Err(err) = fs::write(pidfile, format!("{}", pid)) {
        panic!("write() to pidfile {} failed: {}", pidfile.display(), err);
    }
}

fn ensure_child_reaped(pid: Pid) {
    match kill(pid, None) {
        Ok(_) => {
            warn!("child still running.  terminating.");
            match kill(pid, SIGTERM) {
                Ok(_) => {
                    waitpid(pid, None).unwrap();
                }
                Err(err) => {
                    warn!("failure trying to terminate child: {}", err);
                }
            }
        }
        Err(nix::errno::Errno::ESRCH) => {
            debug!("child no longer exists");
        }
        Err(err) => {
            warn!(
                "Failed to check status of child during failure cleanup: {}",
                err
            );
        }
    }
}

fn init_logger(level: LevelFilter) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "holodekk-subroutine".into(),
        pid: 0,
    };

    let logger = syslog::unix(formatter).expect("could not connect to syslog");
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(level))
        .expect("log::set_boxed_logger() failed");

    panic::set_hook(Box::new(|info| {
        error!("{}", info);
    }));
}
