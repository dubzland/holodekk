use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use crate::libsee;
use crate::pipes::Pipe;

pub struct InputStream(RawFd);

impl Read for InputStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut file = unsafe { File::from_raw_fd(self.0) };
        let res = file.read(buf);
        mem::forget(file); // omit the destruciton of the file, i.e. no call to close(fd).
        res
    }
}

impl Drop for InputStream {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

impl AsRawFd for InputStream {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

pub struct OutputStream(RawFd);

impl Drop for OutputStream {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

pub struct IOMain {
    stdin: Option<OutputStream>,
    stdout: Option<InputStream>,
    stderr: Option<InputStream>,
}

impl IOMain {
    pub fn streams(
        self,
    ) -> (
        Option<OutputStream>,
        Option<InputStream>,
        Option<InputStream>,
    ) {
        (self.stdin, self.stdout, self.stderr)
    }
}

pub struct IOWorker {
    stdin: Option<InputStream>,
    stdout: Option<OutputStream>,
    stderr: Option<OutputStream>,
}

impl IOWorker {
    pub fn streams(
        self,
    ) -> (
        Option<InputStream>,
        Option<OutputStream>,
        Option<OutputStream>,
    ) {
        (self.stdin, self.stdout, self.stderr)
    }
}

pub fn create_pipes(use_stdin: bool, use_stdout: bool, use_stderr: bool) -> (IOMain, IOWorker) {
    let mut main = IOMain {
        stdin: None,
        stdout: None,
        stderr: None,
    };
    let mut worker = IOWorker {
        stdin: None,
        stdout: None,
        stderr: None,
    };

    if use_stdin {
        let stdin = Pipe::new().expect("Failed to create stdin pipe.");
        main.stdin = Some(OutputStream(stdin.wr()));
        worker.stdin = Some(InputStream(stdin.rd()));
        mem::forget(stdin);
    }
    if use_stdout {
        let stdout = Pipe::new().expect("Failed to create stdout pipe.");
        main.stdout = Some(InputStream(stdout.rd()));
        worker.stdout = Some(OutputStream(stdout.wr()));
        mem::forget(stdout);
    }
    if use_stderr {
        let stderr = Pipe::new().expect("Failed to create stderr pipe.");
        main.stderr = Some(InputStream(stderr.rd()));
        worker.stderr = Some(OutputStream(stderr.wr()));
        mem::forget(stderr);
    }

    (main, worker)
}

fn open_dev_null(flags: libc::c_int) -> libsee::Result<RawFd> {
    libsee::open("/dev/null", flags | libc::O_CLOEXEC)
}

pub fn override_streams(
    (ins, outs, errs): (
        Option<InputStream>,
        Option<OutputStream>,
        Option<OutputStream>,
    ),
) -> libsee::Result<()> {
    match ins {
        Some(InputStream(fd)) => {
            libsee::dup2(fd, libc::STDIN_FILENO)?;
        }
        None => {
            let fd = open_dev_null(libc::O_RDONLY)?;
            libsee::dup2(fd, libc::STDIN_FILENO)?;
            libsee::close(fd)?;
        }
    }

    match outs {
        Some(OutputStream(fd)) => {
            libsee::dup2(fd, libc::STDOUT_FILENO)?;
        }
        None => {
            let fd = open_dev_null(libc::O_WRONLY)?;
            libsee::dup2(fd, libc::STDOUT_FILENO)?;
            libsee::close(fd)?;
        }
    }

    match errs {
        Some(OutputStream(fd)) => {
            libsee::dup2(fd, libc::STDERR_FILENO)?;
        }
        None => {
            let fd = open_dev_null(libc::O_WRONLY)?;
            libsee::dup2(fd, libc::STDERR_FILENO)?;
            libsee::close(fd)?;
        }
    }
    Ok(())
}
