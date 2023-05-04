//! Repository watch functionality

use log::warn;
use tokio::sync::broadcast::{error::RecvError, Receiver};

/// unique watch Id
pub type Id = crate::entity::Id;

/// Watch specific errors
pub enum Error {}

/// Handle wrapping a repository watch object
pub struct Handle<T> {
    /// Id of the watch
    pub id: Id,
    /// rx endpoint for receiving watch events
    pub rx: Receiver<T>,
}

impl<T> Handle<T>
where
    T: Clone,
{
    /// Create a new watch handle
    #[must_use]
    pub fn new(id: Id, rx: Receiver<T>) -> Self {
        Self { id, rx }
    }

    /// Future that yields watch events
    pub async fn event(&mut self) -> Option<T> {
        match self.rx.recv().await {
            Ok(msg) => Some(msg),
            Err(RecvError::Closed) => None,
            Err(err) => {
                warn!("Error receiving watch event: {}", err);
                None
            }
        }
    }
}
