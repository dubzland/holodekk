use bytes::Bytes;
use futures::Stream;

use crate::errors::error_chain_fmt;
use crate::images::{SubroutineImage, SubroutineImageId};

#[derive(thiserror::Error)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

trait SubroutineImageStore {
    fn create<S, E>(&self, name: String, stream: S) -> Result<SubroutineImage>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>;
    fn get(&self, id: SubroutineImageId) -> Result<SubroutineImage>;
    fn find(&self) -> Result<Vec<SubroutineImage>>;
    fn delete(&self, id: &SubroutineImageId) -> Result<()>;
}
