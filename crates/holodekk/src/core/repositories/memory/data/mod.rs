mod projectors;
use projectors::*;
mod subroutines;
use subroutines::*;

use std::sync::Arc;

#[derive(Debug)]
pub struct MemoryDatabase {
    projectors: Arc<ProjectorsMemoryStore>,
    subroutines: Arc<SubroutinesMemoryStore>,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self {
            projectors: Arc::new(ProjectorsMemoryStore::default()),
            subroutines: Arc::new(SubroutinesMemoryStore::default()),
        }
    }
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn projectors(&self) -> Arc<ProjectorsMemoryStore> {
        self.projectors.clone()
    }

    pub fn subroutines(&self) -> Arc<SubroutinesMemoryStore> {
        self.subroutines.clone()
    }
}
