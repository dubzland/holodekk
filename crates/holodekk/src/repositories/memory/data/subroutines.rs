use std::{collections::HashMap, sync::RwLock};

use crate::core::subroutines::entities::SubroutineEntity;
use crate::repositories::{RepositoryError, Result};

#[derive(Debug)]
pub struct SubroutinesMemoryStore {
    records: RwLock<HashMap<String, SubroutineEntity>>,
}

impl Default for SubroutinesMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl SubroutinesMemoryStore {
    pub fn add(&self, subroutine: SubroutineEntity) -> Result<()> {
        if self.records.read().unwrap().contains_key(subroutine.id()) {
            Err(RepositoryError::Duplicate(subroutine.id().into()))
        } else {
            self.records
                .write()
                .unwrap()
                .insert(subroutine.id().into(), subroutine);
            Ok(())
        }
    }

    pub fn all(&self) -> Vec<SubroutineEntity> {
        self.records
            .read()
            .unwrap()
            .values()
            .map(|i| i.to_owned())
            .collect()
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.records.write().unwrap().remove(id);
        Ok(())
    }

    pub fn exists(&self, id: &str) -> Result<bool> {
        if self.records.read().unwrap().contains_key(id) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get(&self, id: &str) -> Result<SubroutineEntity> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(RepositoryError::NotFound(id.into()))
        }
    }
}
