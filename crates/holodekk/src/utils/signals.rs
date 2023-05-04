//! OS signal handling utility library.

use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::signal;

/// The kind of signal being monitored
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// Interrupt (SIGNINT)
    Int,
    /// Termination (SIGTERM)
    Term,
    /// Shutdown (SIGQUIT)
    Quit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Int => "SIGINT",
            Kind::Term => "SIGTERM",
            Kind::Quit => "SIGQUIT",
        })
    }
}

/// Simple tokio/futures based signal handler.
pub struct Signals {
    signals: Vec<(Kind, signal::unix::Signal)>,
}

impl Default for Signals {
    fn default() -> Self {
        Self::new()
    }
}

impl Signals {
    /// Creates a handler for the 3 signals listed in `signal::Kind`.
    #[must_use]
    pub fn new() -> Self {
        let signal_map = [
            (signal::unix::SignalKind::interrupt(), Kind::Int),
            (signal::unix::SignalKind::terminate(), Kind::Term),
            (signal::unix::SignalKind::quit(), Kind::Quit),
        ];

        let signals = signal_map
            .iter()
            .filter_map(|(kind, signal)| {
                signal::unix::signal(*kind)
                    .map(|tokio_signal| (*signal, tokio_signal))
                    .map_err(|e| {
                        tracing::error!(
                            "failed to initialize stream handler for signal {:?}, err: {}",
                            signal,
                            e
                        );
                    })
                    .ok()
            })
            .collect();
        Self { signals }
    }
}

impl Future for Signals {
    type Output = Kind;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for (signal, fut) in &mut self.signals {
            if fut.poll_recv(cx).is_ready() {
                return Poll::Ready(*signal);
            }
        }

        Poll::Pending
    }
}
