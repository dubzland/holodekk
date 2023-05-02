mod scenes;
use scenes::*;
mod subroutines;
use subroutines::*;

use std::sync::Arc;

#[derive(Debug)]
pub struct MemoryDatabase {
    scenes: Arc<ScenesMemoryStore>,
    subroutines: Arc<SubroutinesMemoryStore>,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self {
            scenes: Arc::new(ScenesMemoryStore::default()),
            subroutines: Arc::new(SubroutinesMemoryStore::default()),
        }
    }
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn scenes(&self) -> Arc<ScenesMemoryStore> {
        self.scenes.clone()
    }

    pub fn subroutines(&self) -> Arc<SubroutinesMemoryStore> {
        self.subroutines.clone()
    }
}
