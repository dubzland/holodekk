mod entities;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{fmt, result};

use uuid::Uuid;

use holodekk_utils::errors::error_chain_fmt;

use entities::Subroutine;

#[derive(thiserror::Error)]
pub enum Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub type Result<T> = result::Result<T, Error>;

trait SubroutineRepository {
    fn create(&self, subroutine: &Subroutine) -> Result<()>;
    fn update(&self, subroutine: &Subroutine) -> Result<()>;
    fn get_all(&self, name: &str) -> Subroutine;
}

pub struct MemorySubroutineRepository {
    _subroutines: Arc<RwLock<HashMap<Uuid, Subroutine>>>,
}

impl Default for MemorySubroutineRepository {
    fn default() -> Self {
        Self {
            _subroutines: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl MemorySubroutineRepository {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create(&self, _subroutine: &Subroutine) -> Result<()> {
        Ok(())
    }
}
