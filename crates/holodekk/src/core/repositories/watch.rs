use log::warn;
use tokio::sync::broadcast::{error::RecvError, Receiver};

use crate::core::entities::EntityId;

pub type WatchId = EntityId;

pub enum WatchError {}

pub struct WatchHandle<T> {
    pub id: WatchId,
    pub rx: Receiver<T>,
}

impl<T> WatchHandle<T>
where
    T: Clone,
{
    pub fn new(id: WatchId, rx: Receiver<T>) -> Self {
        Self { id, rx }
    }

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
