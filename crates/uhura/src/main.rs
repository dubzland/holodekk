use std::{
    fs::{self, File},
    io::Write,
    os::{fd::RawFd, unix::io::FromRawFd},
    panic,
    path::PathBuf,
    sync::Arc,
};

use clap::Parser;
use log::{debug, error, info, warn, LevelFilter};
use nix::{
    fcntl::{open, OFlag},
    sys::{
        signal::{sigprocmask, SigSet, SigmaskHow, Signal, SIGCHLD, SIGINT, SIGQUIT, SIGTERM},
        stat::Mode,
    },
    unistd::{dup2, fork, setsid, ForkResult, Pid},
};
use serde::Serialize;
use syslog::{BasicLogger, Facility, Formatter3164};

use holodekk::core::entities::{SceneEntityId, SceneName};
use holodekk::utils::{
    libsee,
    signals::{SignalKind, Signals},
    ConnectionInfoError,
};

use uhura::config::UhuraConfig;
use uhura::server::start_uhura_server;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Invalid listener configuration")]
    InvalidListenOptions(#[from] ConnectionInfoError),
    #[error("General IO error occurred")]
    Io(#[from] std::io::Error),
    #[error("General OS error occurred")]
    Nix(#[from] nix::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
struct MessageProjectorPid {
    pid: u32,
}

impl MessageProjectorPid {
    pub fn new(pid: u32) -> Self {
        Self { pid }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// ID of the Scene this projector is running under
    #[arg(long, required = true)]
    id: SceneEntityId,

    /// Name of the Scene is projector is running under
    #[arg(long, required = true)]
    name: SceneName,

    /// Data root path
    #[arg(long, default_value = "/var/lib/holodekk")]
    data_root: PathBuf,

    /// Exec root path
    #[arg(long, default_value = "/run/holodekk")]
    exec_root: PathBuf,

    /// Holodekk bin directory
    #[arg(long, default_value = "/usr/local/bin/")]
    bin_path: PathBuf,

    /// Sync pipe FD
    #[arg(long = "sync-pipe")]
    syncpipe_fd: Option<i32>,
}

fn main() -> Result<()> {
    let options = Options::parse();

    let config = Arc::new(UhuraConfig::new(
        &options.id,
        &options.name,
        &options.data_root,
        &options.exec_root,
        &options.bin_path,
    ));

    // Perform the initial fork
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            libsee::_exit(0);
        }
        Ok(ForkResult::Child) => (),
        Err(err) => {
            error!("Failed to fork from main thres: {}", err);
            libsee::_exit(1);
        }
    }

    init_logger(LevelFilter::Debug);

    info!("uhura coming online with options: {:?}", options);

    // Redirect all streams to /dev/null
    let (dev_null_rd, dev_null_wr) = open_dev_null();
    dup2(dev_null_rd, libsee::STDIN_FILENO).expect("Failed to redirect stdin to /dev/null");
    dup2(dev_null_wr, libsee::STDOUT_FILENO).expect("Failed to redirect stdout to /dev/null");
    dup2(dev_null_wr, libsee::STDERR_FILENO).expect("Failed to redirect stderr to /dev/null");

    // new session
    setsid().expect("Failed to create new session");

    // block signals (until we're ready)
    let mut oldmask = SigSet::empty();
    let signals = signal_mask(&[SIGCHLD, SIGINT, SIGQUIT, SIGTERM]);
    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&signals), Some(&mut oldmask))
        .expect("failed to block signals");

    // fork again
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            write_pidfile(config.scene_paths().pidfile(), child);
            libsee::_exit(1);
        }
        Ok(ForkResult::Child) => {}
        Err(err) => {
            error!("Failed to fork worker: {}", err);
            panic!("fork() of the subroutine process failed: {}", err);
        }
    };

    // re-enable signals
    sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oldmask), None)?;

    // Ensure the root directory exists
    debug!(
        "Checking for existence of root directory: {}",
        config.scene_paths().root().display()
    );
    if !config.scene_paths().root().exists() {
        info!(
            "Creating uhura root directory: {}",
            config.scene_paths().root().display()
        );
        fs::create_dir_all(config.scene_paths().root())
            .expect("Failed to create root directory for uhura");
    }

    main_loop(&options, config.clone())?;

    cleanup(config);
    info!("Shutdown complete.");
    Ok(())
}

#[tokio::main]
async fn main_loop(
    options: &Options,
    config: Arc<UhuraConfig>,
) -> std::result::Result<(), std::io::Error> {
    let uhura_server = start_uhura_server(config.clone());

    // Notify the holodekk of our state
    debug!("Sending status update to parent");
    if options.syncpipe_fd.is_some() {
        send_status_update(options);
    }

    debug!("Complete.  Waiting for shutdown signal");

    let signal = Signals::new().await;
    match signal {
        SignalKind::Int | SignalKind::Term | SignalKind::Quit => {
            debug!("Termination signal received.  Processing shutdown.");

            uhura_server.stop().await.unwrap();
        }
    }
    Ok(())
}

fn cleanup(config: Arc<UhuraConfig>) {
    if config.scene_paths().socket().exists() {
        match std::fs::remove_file(config.scene_paths().socket()) {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to remove projector socket: {}", err);
            }
        }
    }
}

fn send_status_update(options: &Options) {
    let status = MessageProjectorPid::new(std::process::id());
    match serde_json::to_vec(&status) {
        Ok(msg) => {
            let mut sync_pipe = unsafe { File::from_raw_fd(options.syncpipe_fd.unwrap()) };
            match sync_pipe.write_all(&msg) {
                Ok(_) => {}
                Err(err) => {
                    warn!("Failed to write status to sync pipe: {}", err);
                }
            }
        }
        Err(err) => {
            warn!("Failed to serialize JSON for sync update: {}", err);
        }
    }
}

fn write_pidfile(pidfile: &PathBuf, pid: Pid) {
    info!("forked worker with pid: {}", pid);
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
