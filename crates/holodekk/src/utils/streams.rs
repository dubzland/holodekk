use std::os::unix::io::RawFd;

use super::libsee;

#[allow(clippy::missing_errors_doc)]
pub fn open_dev_null(flags: libc::c_int) -> libsee::Result<RawFd> {
    libsee::open("/dev/null", flags | libc::O_CLOEXEC)
}
