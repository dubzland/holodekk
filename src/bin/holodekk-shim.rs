extern crate libc;

use std::panic;
use std::path::{PathBuf};

use log::error;
use syslog::{BasicLogger, Facility, Formatter3164};

use clap::Parser;

use holodekk::shim::Shim;

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
    let pidfile = args.base.join("shim.pid");
    let bundle = args.base.join("bundle");

    setup_logger(log::LevelFilter::Debug);

    let shim = Shim::new(bundle)
        .pid_file(pidfile);
    if args.exec {
        shim.exec(args.name);
    } else {
        shim.create(args.name);
    }

}

fn setup_logger(level: log::LevelFilter) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "holodekk-shim".into(),
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
