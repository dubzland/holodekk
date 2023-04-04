pub use futures_core::future::BoxFuture;

use std::{cell::RefCell, sync::mpsc::Sender};

use crate::server::ServerManager;

use super::TonicServer;

pub struct TonicServerManager {
    thread_handle: RefCell<Option<std::thread::JoinHandle<()>>>,
    shutdown_tx: RefCell<Option<Sender<()>>>,
}

impl ServerManager for TonicServerManager {
    type Server = TonicServer;

    fn new(thread_handle: std::thread::JoinHandle<()>, shutdown_tx: Sender<()>) -> Self {
        Self {
            thread_handle: RefCell::new(Some(thread_handle)),
            shutdown_tx: RefCell::new(Some(shutdown_tx)),
        }
    }

    fn shutdown_tx(&self) -> Option<Sender<()>> {
        self.shutdown_tx.borrow_mut().take()
    }

    fn thread_handle(&self) -> Option<std::thread::JoinHandle<()>> {
        self.thread_handle.borrow_mut().take()
    }
}
