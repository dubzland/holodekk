extern crate libc;

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use holodekk::libsee;
use holodekk::logger::logger_init;
use holodekk::runtime;
use holodekk::streams::override_streams;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the shim's pid file
    #[arg(long = "shim-pidfile", value_name = "file")]
    pidfile: PathBuf,

    /// Path to the runtime (ex. /usr/bin/runc)
    #[arg(long = "runtime-path", value_name = "file")]
    runtime_path: PathBuf,

    /// name for the container instance
    #[arg(long = "container-id", value_name = "id")]
    container_id: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new container
    Create {
        /// Path to the OCI bundle directory
        #[arg(short, long = "bundle", value_name = "dir")]
        bundle_path: PathBuf,
    },
    /// Execute a command in an existing container
    Exec {
        /// Path to the container's pid file
        #[arg(long = "container-pidfile", value_name = "file")]
        container_pidfile: PathBuf,

        runtime_args: Vec<String>,

    }
}

fn main() {
    let options = Cli::parse();

    logger_init("holodekk-shim", log::LevelFilter::Debug);

    let res = libsee::fork().unwrap();
    if let Some(pid) = res {
        // in parent
        write_pid(options.pidfile, pid);
        libsee::_exit(0);
    }

    // In child
    override_streams((None, None, None)).unwrap();
    libsee::setsid().unwrap();
    libsee::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0).unwrap();

    let cmd: Box<dyn runtime::Command> = match &options.command {
        Commands::Create { bundle_path } => {
            Box::new(runtime::CreateCommand::new(bundle_path))
        },
        Commands::Exec { container_pidfile, runtime_args } => {
            Box::new(runtime::ExecCommand::new(container_pidfile, runtime_args))
        },
    };

    runtime::exec(options.runtime_path.as_os_str().to_str().unwrap(), &options.container_id, cmd).unwrap();
}

fn write_pid<F: AsRef<Path>>(pidfile: F, pid: libsee::Pid) {
    if let Err(err) = fs::write(pidfile.as_ref(), format!("{}", pid)) {
        panic!("write() to pidfile {} failed: {}", pidfile.as_ref().display(), err);
    }
}
