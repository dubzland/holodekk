use std::{collections::HashMap, sync::RwLock};

use crate::core::{
    entities::Projector,
    repositories::{memory::MemoryDatabaseKey, Error, Result},
};

#[derive(Debug)]
pub struct ProjectorMemoryStore {
    records: RwLock<HashMap<String, Projector>>,
}

impl Default for ProjectorMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl ProjectorMemoryStore {
    pub fn add(&self, projector: Projector) -> Result<()> {
        if self
            .records
            .read()
            .unwrap()
            .contains_key(&projector.db_key())
        {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(projector.db_key(), projector);
            Ok(())
        }
    }

    pub fn get(&self, id: &str) -> Result<Projector> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn exists(&self, id: &str) -> bool {
        self.records.read().unwrap().contains_key(id)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        if self.records.write().unwrap().remove(id).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound)
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
}
