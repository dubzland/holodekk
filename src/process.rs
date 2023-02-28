use crate::errors::{check_err, Errno};

pub type Pid = libc::pid_t;

pub unsafe fn fork_process() -> Result<Option<Pid>, Errno> {
    let pid = check_err(libc::fork())?;
    if pid == 0 {
        Ok(None)
    } else {
        Ok(Some(pid))
    }
}

pub unsafe fn set_sid() -> Result<(), Errno> {
    check_err(libc::setsid())?;
    Ok(())
}

pub unsafe fn make_subreaper() -> Result<(), Errno> {
    check_err(libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0))?;
    Ok(())
}

