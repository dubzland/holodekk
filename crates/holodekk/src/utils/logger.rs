//! Syslog initialization

use std::panic;

pub use log::{debug, error, LevelFilter};
use syslog::{BasicLogger, Facility, Formatter3164};

/// Initialize logging via the syslog facility.
///
/// # Examples
///
/// ```rust
/// use holodekk::utils::logger;
///
/// logger::init("myapp", log::LevelFilter::Debug);
/// ```
pub fn init(process: &str, level: log::LevelFilter) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: process.to_string(),
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
