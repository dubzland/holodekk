mod scenes;
use scenes::ScenesMemoryStore;
mod subroutines;
use subroutines::SubroutinesMemoryStore;

use std::sync::Arc;

#[derive(Debug)]
pub struct Database {
    scenes: Arc<ScenesMemoryStore>,
    subroutines: Arc<SubroutinesMemoryStore>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            scenes: Arc::new(ScenesMemoryStore::default()),
            subroutines: Arc::new(SubroutinesMemoryStore::default()),
        }
    }
}

impl Database {
    pub fn new() -> Self {
        Database::default()
    }

    pub fn scenes(&self) -> Arc<ScenesMemoryStore> {
        self.scenes.clone()
    }

    pub fn subroutines(&self) -> Arc<SubroutinesMemoryStore> {
        self.subroutines.clone()
    }
}
