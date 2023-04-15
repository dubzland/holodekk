use std::{collections::HashMap, sync::RwLock};

use crate::core::projectors::entities::Projector;
use crate::core::repositories::{Error, RepositoryId, Result};

#[derive(Debug)]
pub struct ProjectorsMemoryStore {
    records: RwLock<HashMap<String, Projector>>,
}

impl Default for ProjectorsMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl ProjectorsMemoryStore {
    pub fn add(&self, projector: Projector) -> Result<()> {
        if self.records.read().unwrap().contains_key(&projector.id()) {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(projector.id(), projector);
            Ok(())
        }
    }

    pub fn all(&self) -> Result<Vec<Projector>> {
        let projectors: Vec<Projector> = self
            .records
            .read()
            .unwrap()
            .values()
            .map(|p| p.to_owned())
            .collect();
        Ok(projectors)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        if self.records.write().unwrap().remove(id).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn exists(&self, id: &str) -> Result<bool> {
        Ok(self.records.read().unwrap().contains_key(id))
    }

    pub fn get(&self, id: &str) -> Result<Projector> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }
}
