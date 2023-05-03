mod scenes;
mod subroutines;

use std::sync::Arc;

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
    pub fn new() -> Self {
        Database::default()
    }

    pub fn scenes(&self) -> Arc<scenes::MemoryStore> {
        self.scenes.clone()
    }

    pub fn subroutines(&self) -> Arc<subroutines::MemoryStore> {
        self.subroutines.clone()
    }
}
