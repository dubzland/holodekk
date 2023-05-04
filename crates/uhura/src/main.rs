use std::{panic, path::PathBuf};

use clap::Parser;
use log::{debug, error, info, warn, LevelFilter};
use nix::{
    sys::signal::{sigprocmask, SigSet, SigmaskHow, Signal, SIGCHLD, SIGINT, SIGQUIT, SIGTERM},
    unistd::{dup2, fork, setsid, ForkResult},
};
use syslog::{BasicLogger, Facility, Formatter3164};

use holodekk::process::{pidfile, syncpipe};
use holodekk::scene;
use holodekk::utils::{
    fs::{ensure_directory, open_dev_null},
    libsee,
    server::Handle,
    signals, Signals,
};

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("General IO error occurred")]
    Io(#[from] std::io::Error),
    #[error("General OS error occurred")]
    Nix(#[from] nix::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// ID of the Scene this projector is running under
    #[arg(long, required = true)]
    id: scene::entity::Id,

    /// Name of the Scene is projector is running under
    #[arg(long, required = true)]
    name: scene::entity::Name,

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

    let config = uhura::Config::new(
        &options.id,
        &options.name,
        &options.data_root,
        &options.exec_root,
        &options.bin_path,
    );

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
    let (dev_null_rd, dev_null_wr) = open_dev_null()?;
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
            pidfile::write_pid(config.scene_paths().pidfile(), child.as_raw())
                .expect("Failed to write pid to pidfile: {err}");
            libsee::_exit(1);
        }
        Ok(ForkResult::Child) => {}
        Err(err) => {
            error!("Failed to fork worker: {err}");
            panic!("fork() of the subroutine process failed: {err}");
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
        ensure_directory(config.scene_paths().root())
            .expect("Failed to create root directory for uhura");
    }

    main_loop(&options, &config)?;

    cleanup(&config);
    info!("Shutdown complete.");
    Ok(())
}

#[tokio::main]
async fn main_loop(
    options: &Options,
    config: &uhura::Config,
) -> std::result::Result<(), std::io::Error> {
    let uhura_server = uhura::start(config);

    // Notify the holodekk of our state
    debug!("Sending status update to parent");
    if let Some(syncpipe_fd) = options.syncpipe_fd {
        if let Err(err) = syncpipe::write_pid(syncpipe_fd, std::process::id() as i32) {
            warn!("Failed to notify parent of our pid: {err}");
        }
    }

    debug!("Complete.  Waiting for shutdown signal");

    let signal = Signals::new().await;
    match signal {
        signals::Kind::Int | signals::Kind::Term | signals::Kind::Quit => {
            debug!("Termination signal received.  Processing shutdown.");

            uhura_server.stop().await.unwrap();
        }
    }
    Ok(())
}

fn cleanup(config: &uhura::Config) {
    if config.scene_paths().socket().exists() {
        match std::fs::remove_file(config.scene_paths().socket()) {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to remove projector socket: {}", err);
            }
        }
    }
}

// fn send_status_update(options: &Options) {
//     let status = MessageProjectorPid::new(std::process::id());
//     match serde_json::to_vec(&status) {
//         Ok(msg) => {
//             let mut sync_pipe = unsafe { File::from_raw_fd(options.syncpipe_fd.unwrap()) };
//             match sync_pipe.write_all(&msg) {
//                 Ok(_) => {}
//                 Err(err) => {
//                     warn!("Failed to write status to sync pipe: {}", err);
//                 }
//             }
//         }
//         Err(err) => {
//             warn!("Failed to serialize JSON for sync update: {}", err);
//         }
//     }
// }

fn signal_mask(signals: &[Signal]) -> SigSet {
    *signals.iter().fold(&mut SigSet::empty(), |mask, sig| {
        mask.add(*sig);
        mask
    })
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
