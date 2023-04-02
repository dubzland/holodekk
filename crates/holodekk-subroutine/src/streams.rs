use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::os::fd::RawFd;
use std::os::unix::io::FromRawFd;
use std::rc::Rc;

use log::warn;

use mio::{event::Source, net::UnixStream, unix::SourceFd, Interest, Registry, Token};

use nix::{
    fcntl::{open, OFlag},
    sys::stat::Mode,
};

const BUF_SIZE: usize = 32 * 1024;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum LogStreamKind {
    Stdout = 1,
    Stderr = 2,
}

pub(crate) struct LogStream {
    fd: RawFd,
    kind: LogStreamKind,
    sinks: HashMap<Token, Rc<RefCell<dyn Write>>>,
}

impl LogStream {
    pub fn new(fd: RawFd, kind: LogStreamKind) -> Self {
        Self {
            fd,
            kind,
            sinks: HashMap::new(),
        }
    }

    fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut file = unsafe { File::from_raw_fd(self.fd) };
        let res = file.read(buf);
        mem::forget(file);
        res
    }

    pub fn scatter(&mut self) -> std::io::Result<usize> {
        let mut buf = [0; BUF_SIZE];
        let nread = self
            .read(&mut buf[1..])
            .expect("log read should have succeeded");

        buf[0] = self.kind as u8;

        if nread > 0 {
            self.sinks.retain(|token, writer| {
                match writer.borrow_mut().write_all(&buf[..nread + 1]) {
                    Ok(_) => true,
                    Err(err) => {
                        warn!("failed to scatter STDIO to sink {:?}: {}", token, err);
                        false
                    }
                }
            });
        }
        Ok(nread)
    }

    pub fn add_sink(&mut self, token: Token, sink: Rc<RefCell<dyn Write>>) {
        self.sinks.insert(token, sink);
    }

    pub fn remove_sink(&mut self, token: Token) {
        self.sinks.remove(&token);
    }
}

impl Source for LogStream {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.fd).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.fd).register(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        SourceFd(&self.fd).deregister(registry)
    }
}

pub(crate) struct StdioSink {
    stream: UnixStream,
    buffer: RefCell<Vec<u8>>,
    buffer_bytes: usize,
}

impl StdioSink {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            buffer: RefCell::new(vec![]),
            buffer_bytes: 0,
        }
    }

    pub fn stream(&mut self) -> &mut UnixStream {
        &mut self.stream
    }

    pub fn data_pending(&self) -> bool {
        !self.buffer_bytes.eq(&0)
    }

    pub fn deliver_data(&mut self) -> std::io::Result<()> {
        let mut buf = self.buffer.borrow_mut();
        let res = self.stream.write_all(&buf[0..self.buffer_bytes]);
        buf.clear();
        self.buffer_bytes = 0;
        res
    }
}

impl Write for StdioSink {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let mut buf = self.buffer.borrow_mut();
        let written = if data.len() + self.buffer_bytes > BUF_SIZE {
            // take as much data as we can
            let available = BUF_SIZE - self.buffer_bytes;
            buf.extend_from_slice(&data[0..available]);
            self.buffer_bytes += available;
            available
        } else {
            buf.extend_from_slice(data);
            self.buffer_bytes += data.len();
            data.len()
        };

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn open_dev_null() -> (RawFd, RawFd) {
    let rd = open(
        "/dev/null",
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect("Opening /dev/null for reading failed");

    let wr = open(
        "/dev/null",
        OFlag::O_WRONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect("Opening /dev/null for writing failed");

    (rd, wr)
}
