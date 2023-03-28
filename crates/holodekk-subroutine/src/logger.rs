use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use chrono::Utc;

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            file: OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap(),
        }
    }

    pub fn write(&mut self, stream: &'static str, buf: &[u8]) -> std::io::Result<()> {
        for line in buf.split(|c| *c == b'\n').filter(|l| !l.is_empty()) {
            let message = format!(
                "{} {} {}\n",
                Utc::now().to_rfc3339(),
                stream,
                String::from_utf8_lossy(line)
            );
            self.file.write_all(message.as_bytes())?;
        }
        Ok(())
    }
}

pub struct Writer {
    logger: Rc<RefCell<Logger>>,
    stream: &'static str,
}

impl Writer {
    pub fn stdout(logger: Rc<RefCell<Logger>>) -> Self {
        Self {
            logger,
            stream: "stdout",
        }
    }
    pub fn stderr(logger: Rc<RefCell<Logger>>) -> Self {
        Self {
            logger,
            stream: "stderr",
        }
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.logger.borrow_mut().write(self.stream, &buf[1..])?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
