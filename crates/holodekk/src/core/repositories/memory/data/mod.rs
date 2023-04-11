mod projectors;
use projectors::*;
mod subroutines;
use subroutines::*;
mod subroutine_instances;
use subroutine_instances::*;

use std::sync::Arc;

#[derive(Debug)]
pub struct MemoryDatabase {
    projectors: Arc<ProjectorMemoryStore>,
    subroutines: Arc<SubroutineMemoryStore>,
    subroutine_instances: Arc<SubroutineInstanceMemoryStore>,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self {
            projectors: Arc::new(ProjectorMemoryStore::default()),
            subroutines: Arc::new(SubroutineMemoryStore::default()),
            subroutine_instances: Arc::new(SubroutineInstanceMemoryStore::default()),
        }
    }
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn projectors(&self) -> Arc<ProjectorMemoryStore> {
        self.projectors.clone()
    }

    pub fn subroutines(&self) -> Arc<SubroutineMemoryStore> {
        self.subroutines.clone()
    }

    pub fn subroutine_instances(&self) -> Arc<SubroutineInstanceMemoryStore> {
        self.subroutine_instances.clone()
    }
}
