use std::ffi::{CStr, CString};

pub use libc::{c_char, c_int, c_ulong};

pub use libc::{
    PR_SET_CHILD_SUBREAPER, PR_SET_PDEATHSIG, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(Error{ errno: errno() })
        } else {
            Ok(res)
        }
    }};
}

pub type Pid = libc::pid_t;

pub type Result<T> = std::result::Result<T, Error>;
pub type Errno = libc::c_int;

#[derive(Debug)]
pub struct Error {
    errno: Errno,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_desc: *const libc::c_char = unsafe { libc::strerror(self.errno) };
        let c_str_desc: &CStr = unsafe { CStr::from_ptr(c_desc) };
        let c_slice_desc: &str = c_str_desc.to_str().unwrap();
        f.write_str(c_slice_desc)?;
        write!(f, ", errno {}", self.errno)?;
        Ok(())
    }
}

impl std::error::Error for Error {}

pub trait Num {
    fn is_err(&self) -> bool;
}

impl Num for i8 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for i16 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for i32 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for i64 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for isize {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

fn errno() -> Errno {
    std::io::Error::last_os_error()
        .raw_os_error()
        .expect("errno")
}

pub fn close(fd: c_int) -> Result<()> {
    syscall!(close(fd))?;
    Ok(())
}

pub fn dup2(fd_new: c_int, fd_old: c_int) -> Result<()> {
    syscall!(dup2(fd_new, fd_old))?;
    Ok(())
}

pub fn execv(argv: &[CString]) {
    let mut argv_raw: Vec<*const c_char> = vec![];
    for arg in argv.iter() {
        argv_raw.push(arg.as_ptr())
    }
    argv_raw.push(std::ptr::null());
    unsafe {
        libc::execv(argv_raw[0], argv_raw.as_ptr());
    }
}

pub fn _exit(code: c_int) -> ! {
    unsafe {
        libc::_exit(code);
    }
}

pub fn fork() -> Result<Option<Pid>> {
    match syscall!(fork())? {
        0 => Ok(None),
        pid => Ok(Some(pid)),
    }
}

pub fn open(path: &str, flags: c_int) -> Result<c_int> {
    let path_c = CString::new(path).unwrap();
    syscall!(open(path_c.as_ptr(), flags))
}

pub fn prctl(
    option: c_int,
    arg2: c_ulong,
    arg3: c_ulong,
    arg4: c_ulong,
    arg5: c_ulong,
) -> Result<()> {
    syscall!(prctl(option, arg2, arg3, arg4, arg5))?;
    Ok(())
}

pub fn setsid() -> Result<()> {
    syscall!(setsid())?;
    Ok(())
}

pub fn waitpid(pid: Pid, status: &mut c_int, flags: c_int) -> Result<()> {
    syscall!(waitpid(pid, status, flags))?;
    Ok(())
}
