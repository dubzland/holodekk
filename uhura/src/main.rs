use std::fs;
use std::os::fd::{AsRawFd, RawFd};
use std::panic;
use std::path::PathBuf;

use clap::Parser;

use log::{debug, error, LevelFilter};

use mio::{unix::SourceFd, Events, Interest, Poll, Token};

use nix::{
    fcntl::{open, OFlag},
    sys::{
        signal::{sigprocmask, SigSet, SigmaskHow, Signal, SIGCHLD, SIGINT, SIGQUIT, SIGTERM},
        signalfd::SignalFd,
        stat::Mode,
    },
    unistd::{dup2, fork, setsid, ForkResult, Pid},
};

use syslog::{BasicLogger, Facility, Formatter3164};

mod projector;

// TODO: Get rid of this
use holodekk::utils::libsee::prctl;
use holodekk_projector::Result;
use projector::Projector;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// Namespace this projector is responsible for
    #[arg(long, short, required = true)]
    namespace: String,

    /// Path to the projector's pid file
    #[arg(long, value_name = "file", required = true)]
    pidfile: PathBuf,

    /// Projector port
    #[arg(short, long, required = true)]
    port: String,

    /// RPC port
    #[arg(long = "rpc-port", required = true)]
    rpc_port: u16,
}

fn main() -> Result<()> {
    let options = Options::parse();

    // Perform the initial fork
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            unsafe { libc::_exit(0) };
        }
        Ok(ForkResult::Child) => (),
        Err(_) => {
            eprintln!("Fork failed");
            unsafe { libc::_exit(1) };
        }
    }

    init_logger(LevelFilter::Debug);

    // Redirect all streams to /dev/null
    let (dev_null_rd, dev_null_wr) = open_dev_null();
    dup2(dev_null_rd, libc::STDIN_FILENO).expect("Failed to redirect stdin to /dev/null");
    dup2(dev_null_wr, libc::STDOUT_FILENO).expect("Failed to redirect stdout to /dev/null");
    dup2(dev_null_wr, libc::STDERR_FILENO).expect("Failed to redirect stderr to /dev/null");

    // new session
    setsid().expect("Failed to create new session");

    // make us a subreaper
    prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0).expect("Unable to set ourselves as subreaper");

    // block signals (until we're ready)
    let mut oldmask = SigSet::empty();
    let signals = signal_mask(&[SIGCHLD, SIGINT, SIGQUIT, SIGTERM]);
    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&signals), Some(&mut oldmask))
        .expect("failed to block signals");

    // fork again
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            debug!("Second fork complete.  Child pid: {}", child);
            write_pidfile(&options.pidfile, child);
            unsafe { libc::_exit(1) };
        }
        Ok(ForkResult::Child) => {}
        Err(err) => {
            error!("Fork failed");
            panic!("fork() of the subroutine process failed: {}", err);
        }
    };

    // Start a projector
    let projector = Projector::build()
        .for_namespace(&options.namespace)
        .with_docker_engine()
        .build();

    let (admin_port, subroutine_port) = projector.start()?;
    debug!(
        "Projector running on ports {}/{}",
        admin_port, subroutine_port
    );

    main_loop()?;

    debug!("About to exit");
    projector.stop()?;
    Ok(())
}

fn main_loop() -> std::io::Result<()> {
    let mut sigset = SigSet::empty();
    sigset.add(SIGINT);
    sigset.add(SIGQUIT);
    sigset.add(SIGTERM);
    let mut signal_fd = SignalFd::new(&sigset)?;

    let mut poll = Poll::new()?;
    poll.registry().register(
        &mut SourceFd(&signal_fd.as_raw_fd()),
        Token(0),
        Interest::READABLE,
    )?;

    let mut events = Events::with_capacity(1024);

    debug!("entering main poll loop");
    loop {
        debug!("loop");
        poll.poll(&mut events, None)?;
        debug!("poll returned");

        for event in &events {
            if event.token() == Token(0) && event.is_readable() {
                // Received a signal.  see what it is.
                match signal_fd.read_signal() {
                    Ok(Some(sinfo)) => Signal::try_from(sinfo.ssi_signo as libc::c_int),
                    Ok(None) => panic!("signal fired, but nothing was available"),
                    Err(err) => panic!("read(signalfd) failed {}", err),
                }?;

                // If we're here, we got one of SIGINT, SIGQUIT, or SIGTERM
                return Ok(());
            }
        }
    }
}

fn write_pidfile(pidfile: &PathBuf, pid: Pid) {
    debug!("forked worker with pid: {}", pid);
    if let Err(err) = fs::write(pidfile, format!("{}", pid)) {
        panic!("write() to pidfile {} failed: {}", pidfile.display(), err);
    }
}

fn signal_mask(signals: &[Signal]) -> SigSet {
    *signals.iter().fold(&mut SigSet::empty(), |mask, sig| {
        mask.add(*sig);
        mask
    })
}

fn open_dev_null() -> (RawFd, RawFd) {
    let rd = open(
        "/dev/null",
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect("Opening /dev/null for reading failed");

    let wr = open(
        "/dev/null",
        OFlag::O_WRONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect("Opening /dev/null for writing failed");

    (rd, wr)
}

fn init_logger(level: LevelFilter) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "uhura".into(),
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
