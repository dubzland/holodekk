use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::signal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalKind {
    Int,
    Term,
    Quit,
}

impl fmt::Display for SignalKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SignalKind::Int => "SIGINT",
            SignalKind::Term => "SIGTERM",
            SignalKind::Quit => "SIGQUIT",
        })
    }
}

pub struct Signals {
    signals: Vec<(SignalKind, signal::unix::Signal)>,
}

impl Default for Signals {
    fn default() -> Self {
        Self::new()
    }
}

impl Signals {
    pub fn new() -> Self {
        let signal_map = [
            (signal::unix::SignalKind::interrupt(), SignalKind::Int),
            (signal::unix::SignalKind::terminate(), SignalKind::Term),
            (signal::unix::SignalKind::quit(), SignalKind::Quit),
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
                        )
                    })
                    .ok()
            })
            .collect();
        Self { signals }
    }
}

impl Future for Signals {
    type Output = SignalKind;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for (signal, fut) in self.signals.iter_mut() {
            if fut.poll_recv(cx).is_ready() {
                return Poll::Ready(*signal);
            }
        }

        Poll::Pending
    }
}
