pub type Errno = libc::c_int;

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

pub fn check_err<N: Num>(ret: N) -> Result<N, Errno> {
    if ret.is_err() {
        Err(errno())
    } else {
        Ok(ret)
    }
}

fn errno() -> Errno {
    std::io::Error::last_os_error()
        .raw_os_error()
        .expect("errno")
}
