use std::os::fd::AsRawFd;

use log::{debug, warn};
use mio::{event::Source, unix::SourceFd, Interest, Registry, Token};

use nix::{
    sys::{
        signal::{self, SigSet, Signal},
        signalfd,
        wait::{waitpid, WaitPidFlag, WaitStatus},
    },
    unistd::Pid,
};

type ExitCode = i32;

#[derive(Copy, Clone, Debug)]
pub enum ExitStatus {
    Normal(Pid, ExitCode),
    Signaled(Pid, Signal),
}

/// Processes signals bound for the shim.
///
/// Also monitors for SIGCHLD signals generated by the subroutine.
#[derive(Debug)]
pub struct SignalHandler {
    child_pid: Pid,
    fd: signalfd::SignalFd,
    status: Option<ExitStatus>,
}

impl SignalHandler {
    pub fn new(child_pid: Pid, signals: &SigSet) -> Self {
        Self {
            child_pid,
            fd: signalfd::SignalFd::new(signals).expect("Could not create a signal set"),
            status: None,
        }
    }

    /// Called by the event loop when notified of pending signals.
    ///
    /// Signals received here fall into one of two categories:
    /// - SIGCHLD: our monitored child (subroutine) changed state (probably exit)
    /// - Others: these are signals received by US that need to be forwarded to the subroutine
    pub fn handle_signal(&mut self) -> nix::Result<()> {
        match self.read_signal()? {
            Signal::SIGCHLD => self.handle_sigchld()?,
            other => self.forward_signal(other)?,
        };
        Ok(())
    }

    /// Returns the exit status of our monitored subroutine (if exited), or None
    pub fn status(&self) -> Option<ExitStatus> {
        self.status
    }

    /// Attempts to read the actual signal from the underlying OS.
    fn read_signal(&mut self) -> nix::Result<Signal> {
        match self.fd.read_signal() {
            Ok(Some(sinfo)) => Signal::try_from(sinfo.ssi_signo as libc::c_int),
            Ok(None) => panic!("wtf? We are in blocking mode"),
            Err(err) => panic!("read(signalfd) failed {}", err),
        }
    }

    fn handle_sigchld(&mut self) -> nix::Result<()> {
        match waitpid(self.child_pid, Some(WaitPidFlag::WNOHANG))? {
            WaitStatus::Exited(pid, code) => {
                assert!(pid == self.child_pid);
                self.status = Some(ExitStatus::Normal(pid, code));
            }
            WaitStatus::Signaled(pid, sig, _) => {
                assert!(pid == self.child_pid);
                self.status = Some(ExitStatus::Signaled(pid, sig));
            }
            _ => {
                warn!("SIGCHLD fired, but nothing interesting to report");
            }
        }
        Ok(())
    }

    fn forward_signal(&self, signal: Signal) -> nix::Result<()> {
        debug!(
            "Forwarding signal {} to child process {}",
            signal, self.child_pid
        );

        signal::kill(self.child_pid, signal)
    }
}

impl Source for SignalHandler {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.fd.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.fd.as_raw_fd()).register(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        SourceFd(&self.fd.as_raw_fd()).deregister(registry)
    }
}

pub fn signal_mask(signals: &[Signal]) -> SigSet {
    *signals.iter().fold(&mut SigSet::empty(), |mask, sig| {
        mask.add(*sig);
        mask
    })
}
