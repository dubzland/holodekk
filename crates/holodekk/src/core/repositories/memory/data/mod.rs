mod projectors;
use projectors::*;
mod subroutines;
use subroutines::*;
mod subroutine_instances;
use subroutine_instances::*;

use std::sync::Arc;

#[derive(Debug)]
pub struct MemoryDatabase {
    projectors: Arc<ProjectorsMemoryStore>,
    subroutines: Arc<SubroutinesMemoryStore>,
    subroutine_instances: Arc<SubroutineInstancesMemoryStore>,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self {
            projectors: Arc::new(ProjectorsMemoryStore::default()),
            subroutines: Arc::new(SubroutinesMemoryStore::default()),
            subroutine_instances: Arc::new(SubroutineInstancesMemoryStore::default()),
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

    pub fn subroutine_instances(&self) -> Arc<SubroutineInstancesMemoryStore> {
        self.subroutine_instances.clone()
    }
}
