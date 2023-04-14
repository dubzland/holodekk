mod projectors;
use projectors::*;
mod subroutines;
use subroutines::*;
mod subroutine_definitions;
use subroutine_definitions::*;

use std::sync::Arc;

#[derive(Debug)]
pub struct MemoryDatabase {
    projectors: Arc<ProjectorsMemoryStore>,
    subroutines: Arc<SubroutinesMemoryStore>,
    subroutine_definitions: Arc<SubroutineDefinitionsMemoryStore>,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self {
            projectors: Arc::new(ProjectorsMemoryStore::default()),
            subroutines: Arc::new(SubroutinesMemoryStore::default()),
            subroutine_definitions: Arc::new(SubroutineDefinitionsMemoryStore::default()),
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

    pub fn subroutine_definitions(&self) -> Arc<SubroutineDefinitionsMemoryStore> {
        self.subroutine_definitions.clone()
    }
}
