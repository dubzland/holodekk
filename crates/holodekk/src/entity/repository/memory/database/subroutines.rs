use std::{collections::HashMap, sync::RwLock};

use crate::entity::repository::{Error, Result};
use crate::subroutine::{entity::Id, Entity};

#[derive(Debug)]
pub struct MemoryStore {
    records: RwLock<HashMap<Id, Entity>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl MemoryStore {
    pub fn add(&self, subroutine: Entity) -> Result<()> {
        if self.records.read().unwrap().contains_key(&subroutine.id) {
            Err(Error::Conflict(format!(
                "Subroutine already exists with id {}",
                subroutine.id
            )))
        } else {
            self.records
                .write()
                .unwrap()
                .insert(subroutine.id.clone(), subroutine);
            Ok(())
        }
    }

    pub fn all(&self) -> Vec<Entity> {
        self.records
            .read()
            .unwrap()
            .values()
            .map(std::clone::Clone::clone)
            .collect()
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn delete(&self, id: &Id) -> Result<()> {
        self.records.write().unwrap().remove(id);
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn exists(&self, id: &Id) -> Result<bool> {
        if self.records.read().unwrap().contains_key(id) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get(&self, id: &Id) -> Result<Entity> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.clone())
        } else {
            Err(Error::NotFound(id.clone()))
        }
    }
}
