use log::warn;
use tokio::sync::broadcast::{error::RecvError, Receiver};

pub type Id = crate::entity::Id;

pub enum Error {}

pub struct Handle<T> {
    pub id: Id,
    pub rx: Receiver<T>,
}

impl<T> Handle<T>
where
    T: Clone,
{
    pub fn new(id: Id, rx: Receiver<T>) -> Self {
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
