use std::{
    fs::{self, File},
    io::Write,
    net::Ipv4Addr,
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

use holodekk::{
    repositories::{memory::MemoryRepository, Repository},
    servers::{ProjectorServer, UhuraServer},
    utils::{
        // fs::cleanup as cleanup_socket,
        libsee,
        signals::{SignalKind, Signals},
        ConnectionInfo,
        ConnectionInfoError,
    },
    HolodekkConfig,
};

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
struct MessageProjectorPid<'a> {
    pid: u32,
    projector_port: Option<u16>,
    projector_address: Option<Ipv4Addr>,
    projector_socket: Option<&'a PathBuf>,
    uhura_port: Option<u16>,
    uhura_address: Option<Ipv4Addr>,
    uhura_socket: Option<&'a PathBuf>,
}

impl<'a> MessageProjectorPid<'a> {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            projector_port: None,
            projector_address: None,
            projector_socket: None,
            uhura_port: None,
            uhura_address: None,
            uhura_socket: None,
        }
    }

    pub fn with_projector_listener(
        &mut self,
        port: Option<u16>,
        address: Option<Ipv4Addr>,
        socket: Option<&'a PathBuf>,
    ) -> &mut Self {
        self.projector_port = port;
        self.projector_address = address;
        self.projector_socket = socket.to_owned();
        self
    }

    /// Assigns the admin attributes for this status update.
    ///
    /// # Arguments
    ///
    /// `port` - Port number we are listening on
    /// `address` - IPV4 address
    /// `socket` - Unix socket path
    pub fn with_uhura_listener(
        &mut self,
        port: Option<u16>,
        address: Option<Ipv4Addr>,
        socket: Option<&'a PathBuf>,
    ) -> &mut Self {
        self.uhura_port = port;
        self.uhura_address = address;
        self.uhura_socket = socket.to_owned();
        self
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// Fleet this projector belongs to
    #[arg(long)]
    fleet: String,

    /// Namespace this projector is responsible for
    #[arg(long, short, required = true)]
    namespace: String,

    /// Path to the projector's pid file
    #[arg(long, value_name = "file", required = true)]
    pidfile: PathBuf,

    /// Root directory
    #[arg(long = "sync-pipe")]
    projector_root: PathBuf,

    /// Projector port
    #[arg(short, long)]
    projector_port: Option<u16>,

    /// Projector listen address (IP)
    #[arg(long)]
    projector_address: Option<Ipv4Addr>,

    /// Projector listen socket (UDS)
    #[arg(long, conflicts_with_all = ["projector_port", "projector_address"])]
    projector_socket: Option<PathBuf>,

    /// Uhura API port
    #[arg(long)]
    uhura_port: Option<u16>,

    /// Uhura API listen address (IP)
    #[arg(long)]
    uhura_address: Option<Ipv4Addr>,

    /// Uhura API listen socket (UDS)
    #[arg(long, conflicts_with_all = ["uhura_port", "uhura_address"])]
    uhura_socket: Option<PathBuf>,

    /// Sync pipe FD
    #[arg(long = "sync-pipe")]
    syncpipe_fd: Option<i32>,

    /// Holodekk root directory
    #[arg(long, default_value = "/var/lib/holodekkd")]
    holodekk_root: PathBuf,

    /// Holodekk bin directory
    #[arg(long, default_value = "/usr/local/bin/")]
    holodekk_bin: PathBuf,
}

fn main() -> Result<()> {
    let options = Options::parse();

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
            write_pidfile(&options.pidfile, child);
            libsee::_exit(1);
        }
        Ok(ForkResult::Child) => {}
        Err(err) => {
            error!("Failed to fork worker: {}", err);
            panic!("fork() of the subroutine process failed: {}", err);
        }
    };

    let config = Arc::new(HolodekkConfig {
        fleet: options.fleet.to_owned(),
        root_path: options.holodekk_root.to_owned(),
        bin_path: options.holodekk_bin.to_owned(),
    });

    // build the servers
    let repo = Arc::new(MemoryRepository::default());
    let uhura_server = UhuraServer::new(config, &options.namespace, repo.clone());
    let uhura_listener_config = ConnectionInfo::from_options(
        options.uhura_port.as_ref(),
        options.uhura_address.as_ref(),
        options.uhura_socket.as_ref(),
    )
    .unwrap();
    let projector_server = ProjectorServer::new(&options.fleet, &options.namespace, repo);
    let projector_listener_config = ConnectionInfo::from_options(
        options.projector_port.as_ref(),
        options.projector_address.as_ref(),
        options.projector_socket.as_ref(),
    )
    .unwrap();

    // re-enable signals
    sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oldmask), None)?;

    main_loop(
        &options,
        uhura_server,
        uhura_listener_config,
        projector_server,
        projector_listener_config,
    )?;

    cleanup(&options);
    info!("Shutdown complete.");
    Ok(())
}

#[tokio::main]
async fn main_loop<T>(
    options: &Options,
    mut uhura_server: UhuraServer<T>,
    uhura_config: ConnectionInfo,
    mut projector_server: ProjectorServer<T>,
    projector_config: ConnectionInfo,
) -> std::result::Result<(), std::io::Error>
where
    T: Repository,
{
    uhura_server.start(uhura_config);
    projector_server.start(projector_config);

    // Notify the holodekk of our state
    debug!("Sending status update to parent");
    if options.syncpipe_fd.is_some() {
        send_status_update(options);
    }

    let signal = Signals::new().await;
    match signal {
        SignalKind::Int => {
            debug!("SIGINT received.  Processing shutdown.");

            uhura_server.stop().await;
            projector_server.stop().await;
        }
        SignalKind::Quit | SignalKind::Term => {
            debug!("Unexpected {} received.  Terminating immediately", signal);
        }
    }
    Ok(())
}

fn cleanup(options: &Options) {
    if options.uhura_socket.is_some() {
        let admin_socket = options.uhura_socket.as_ref().unwrap();
        if admin_socket.exists() {
            match std::fs::remove_file(admin_socket) {
                Ok(_) => {}
                Err(err) => {
                    warn!("Failed to remove admin socket: {}", err);
                }
            }
        }
    }

    if options.projector_socket.is_some() {
        let projector_socket = options.projector_socket.as_ref().unwrap();
        if projector_socket.exists() {
            match std::fs::remove_file(projector_socket) {
                Ok(_) => {}
                Err(err) => {
                    warn!("Failed to remove projector socket: {}", err);
                }
            }
        }
    }
}

fn send_status_update(options: &Options) {
    let mut status = MessageProjectorPid::new(std::process::id());
    status.with_uhura_listener(
        options.uhura_port,
        options.uhura_address,
        options.uhura_socket.as_ref(),
    );
    status.with_projector_listener(
        options.projector_port,
        options.projector_address,
        options.projector_socket.as_ref(),
    );
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
