//! Basic "database" using in-memory storage

mod scenes;
mod subroutines;

use std::sync::Arc;

/// Bare-bones "database"
#[derive(Debug)]
pub struct Database {
    scenes: Arc<scenes::MemoryStore>,
    subroutines: Arc<subroutines::MemoryStore>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            scenes: Arc::new(scenes::MemoryStore::default()),
            subroutines: Arc::new(subroutines::MemoryStore::default()),
        }
    }
}

impl Database {
    /// initialize a new database
    #[must_use]
    pub fn new() -> Self {
        Database::default()
    }

    /// access the `scene` data store
    #[must_use]
    pub fn scenes(&self) -> Arc<scenes::MemoryStore> {
        self.scenes.clone()
    }

    /// access the `subroutine` data store
    #[must_use]
    pub fn subroutines(&self) -> Arc<subroutines::MemoryStore> {
        self.subroutines.clone()
    }
}
