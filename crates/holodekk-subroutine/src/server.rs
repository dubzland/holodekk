use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Result};
use std::os::fd::RawFd;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use log::{debug, warn};

use mio::net::UnixListener;
use mio::{event::Event, Events, Interest, Poll, Token};

use nix::{
    sys::signal::{SIGCHLD, SIGINT, SIGQUIT, SIGTERM},
    unistd::Pid,
};

use super::logger::{Logger, Writer};
use super::signals::{signal_mask, ExitStatus, SignalHandler};
use super::streams::{LogStream, LogStreamKind, StdioSink};

const TOKEN_SIGNAL: Token = Token(0);
const TOKEN_STDOUT: Token = Token(1);
const TOKEN_STDERR: Token = Token(2);
const TOKEN_ATTACH: Token = Token(3);
const TOKEN_UNUSED: Token = Token(100);

pub struct ServerBuilder {
    signal_handler: Option<SignalHandler>,
    stdout_scatterer: Option<LogStream>,
    stderr_scatterer: Option<LogStream>,
    logger: Option<Rc<RefCell<Logger>>>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            signal_handler: None,
            stdout_scatterer: None,
            stderr_scatterer: None,
            logger: None,
        }
    }

    pub fn with_child(self, pid: Pid) -> Self {
        let signals = signal_mask(&[SIGCHLD, SIGINT, SIGQUIT, SIGTERM]);
        let signal_handler = SignalHandler::new(pid, &signals);

        Self {
            signal_handler: Some(signal_handler),
            ..self
        }
    }

    pub fn with_stdio(self, stdout: RawFd, stderr: RawFd) -> Self {
        Self {
            stdout_scatterer: Some(LogStream::new(stdout, LogStreamKind::Stdout)),
            stderr_scatterer: Some(LogStream::new(stderr, LogStreamKind::Stderr)),
            ..self
        }
    }

    pub fn with_log_file(self, logfile: &PathBuf) -> Self {
        // setup the log
        let logger = Rc::new(RefCell::new(Logger::new(logfile)));

        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn listen_uds(self, log_socket: &PathBuf) -> Result<Server> {
        // clean up if necessary
        if log_socket.exists() {
            std::fs::remove_file(log_socket).expect("Failed to remove existing listening socket");
        }

        let logger = self.logger.unwrap();
        let mut signal_handler = self.signal_handler.unwrap();
        let mut stdout_scatterer = self.stdout_scatterer.unwrap();
        let mut stderr_scatterer = self.stderr_scatterer.unwrap();

        // attach the log to the streams
        let log_token = Token(4);
        stdout_scatterer.add_sink(
            log_token,
            Rc::new(RefCell::new(Writer::stdout(logger.clone()))),
        );
        stderr_scatterer.add_sink(log_token, Rc::new(RefCell::new(Writer::stderr(logger))));

        // create the watcher
        let poll = Poll::new()?;

        // register our base emitters
        poll.registry()
            .register(&mut signal_handler, TOKEN_SIGNAL, Interest::READABLE)?;

        poll.registry()
            .register(&mut stdout_scatterer, TOKEN_STDOUT, Interest::READABLE)?;

        poll.registry()
            .register(&mut stderr_scatterer, TOKEN_STDERR, Interest::READABLE)?;

        // Start the listener
        let mut attach_listener = UnixListener::bind(log_socket)?;

        poll.registry()
            .register(&mut attach_listener, TOKEN_ATTACH, Interest::READABLE)?;

        Ok(Server::new(
            poll,
            signal_handler,
            stdout_scatterer,
            stderr_scatterer,
            attach_listener,
        ))
    }
}

pub struct Server {
    poll: Poll,
    signal_handler: SignalHandler,
    stdout_scatterer: LogStream,
    stderr_scatterer: LogStream,
    attach_listener: UnixListener,

    timeout: Duration,
    log_sinks: HashMap<Token, Rc<RefCell<StdioSink>>>,
    unique_token: Token,
}

impl Server {
    pub fn build() -> ServerBuilder {
        ServerBuilder::new()
    }

    fn new(
        poll: Poll,
        signal_handler: SignalHandler,
        stdout_scatterer: LogStream,
        stderr_scatterer: LogStream,
        attach_listener: UnixListener,
    ) -> Self {
        Self {
            poll,
            signal_handler,
            stdout_scatterer,
            stderr_scatterer,
            attach_listener,
            log_sinks: HashMap::new(),
            timeout: Duration::from_secs(5),
            unique_token: TOKEN_UNUSED,
        }
    }

    pub fn run(&mut self) -> Result<ExitStatus> {
        while self.signal_handler.status().is_none() {
            self.poll_once()?;
        }

        // Process is complete.  Drain stdio
        self.poll.registry().deregister(&mut self.signal_handler)?;
        self.poll.registry().deregister(&mut self.attach_listener)?;
        self.timeout = Duration::from_millis(0);

        while self.poll_once()? != 0 {
            debug!("draining subroutine logs");
        }

        Ok(self.signal_handler.status().unwrap())
    }

    fn poll_once(&mut self) -> Result<i32> {
        let mut events = Events::with_capacity(128);

        self.poll.poll(&mut events, Some(self.timeout))?;

        let mut event_count = 0;

        for event in events.iter() {
            event_count += 1;
            match event.token() {
                TOKEN_SIGNAL => self.signal_handler.handle_signal()?,
                TOKEN_ATTACH => {
                    self.handle_attach_event(event)?;
                }
                TOKEN_STDOUT => {
                    self.handle_stdout_event(event)?;
                }
                TOKEN_STDERR => {
                    self.handle_stderr_event(event)?;
                }
                _ => {
                    let done = self.handle_sink_event(event)?;
                    if done {
                        let token = event.token();
                        self.drop_sink(token)?;
                    }
                }
            }
        }

        // Neet to roll through the log sinks and flip any with pending data to Interest::WRITABLE
        for (token, sink) in self.log_sinks.iter() {
            if sink.borrow().data_pending() {
                self.poll.registry().reregister(
                    sink.borrow_mut().stream(),
                    *token,
                    Interest::READABLE.add(Interest::WRITABLE),
                )?;
            }
        }

        Ok(event_count)
    }

    fn handle_attach_event(&mut self, event: &Event) -> Result<()> {
        // Ensure the socket is actually readable
        if event.is_readable() {
            // accept the connection
            let (mut connection, address) = self.attach_listener.accept()?;

            // Register the connection
            let token = self.next_token();
            self.poll
                .registry()
                .register(&mut connection, token, Interest::READABLE)?;

            // create a log sink
            let sink = Rc::new(RefCell::new(StdioSink::new(connection)));
            self.stdout_scatterer.add_sink(token, sink.clone());
            self.stderr_scatterer.add_sink(token, sink.clone());
            self.log_sinks.insert(token, sink);
            debug!("Accepted a connection from {:?}", address);
        } else {
            warn!("Attach event triggered, but socket is not readable.");
        }

        Ok(())
    }

    fn handle_stdout_event(&mut self, event: &Event) -> Result<usize> {
        if event.is_read_closed() {
            debug!("stdout closed");
            Ok(0)
        } else if event.is_readable() {
            self.stdout_scatterer.scatter()
        } else {
            warn!("stdout triggered an event, but nothing interesting available.");
            Ok(0)
        }
    }

    fn handle_stderr_event(&mut self, event: &Event) -> Result<usize> {
        if event.is_read_closed() {
            debug!("stderr closed");
            Ok(0)
        } else if event.is_readable() {
            self.stderr_scatterer.scatter()
        } else {
            warn!("stderr triggered an event, but was not readable.");
            Ok(0)
        }
    }

    fn handle_sink_event(&mut self, event: &Event) -> Result<bool> {
        let token = event.token();

        // make sure we actually have this sink
        if !self.log_sinks.contains_key(&token) {
            warn!("handle_sink_event() fired for non-existent sink.");
            return Ok(false);
        }

        // so that this dosn't panic
        let sink = self.log_sinks.get(&token).unwrap();
        let mut sink = sink.borrow_mut();
        if event.is_readable() {
            // Probably a closed socket
            let mut buf: [u8; 1] = [0; 1];
            match sink.stream().read(&mut buf) {
                Ok(num_read) => {
                    if (num_read) > 0 {
                        warn!("Received data from log sink unexpectedly.");
                    } else {
                        debug!("log sink disconnect");
                    }
                }
                Err(err) => {
                    warn!("Error reading from log sink: {}", err);
                }
            }

            // let the poll know we are done with this sink
            Ok(true)
        } else if event.is_writable() && sink.data_pending() {
            match sink.deliver_data() {
                Ok(_) => {
                    self.poll
                        .registry()
                        .reregister(sink.stream(), token, Interest::READABLE)?;
                    Ok(false)
                }
                Err(err) => {
                    warn!("Error delivering data for sink: {}", err);
                    // drop this sink
                    Ok(true)
                }
            }
        } else {
            warn!("log sink in strange state.  dropping.");
            Ok(true)
        }
    }

    fn drop_sink(&mut self, token: Token) -> Result<()> {
        // make sure we actually have this sink
        if !self.log_sinks.contains_key(&token) {
            warn!("handle_sink_event() fired for non-existent sink.");
            return Ok(());
        }

        // so that this doesn't panic
        let sink = self.log_sinks.get(&token).unwrap().clone();
        self.poll
            .registry()
            .deregister(sink.borrow_mut().stream())?;
        self.stdout_scatterer.remove_sink(token);
        self.stderr_scatterer.remove_sink(token);
        self.log_sinks.remove(&token);

        Ok(())
    }

    fn next_token(&mut self) -> Token {
        let next = self.unique_token.0;
        self.unique_token.0 += 1;
        Token(next)
    }
}
