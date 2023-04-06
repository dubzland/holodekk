use std::os::fd::RawFd;

use libc::c_int;

pub struct Pipe {
    rd: RawFd,
    wr: RawFd,
}

impl Pipe {
    pub fn new() -> Option<Self> {
        let mut fds: [c_int; 2] = [-1, -1];
        let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };

        if ret == -1 {
            None
        } else {
            Some(Pipe {
                rd: fds[0],
                wr: fds[1],
            })
        }
    }

    pub fn rd(&self) -> RawFd {
        self.rd
    }
    pub fn wr(&self) -> RawFd {
        self.wr
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.rd);
            libc::close(self.wr);
        }
    }
}

// pub fn create_pipes(use_stdin: bool, use_stdout: bool, use_stderr: bool)
