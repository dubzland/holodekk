use std::{collections::HashMap, sync::RwLock};

use log::debug;

use crate::core::projectors::entities::ProjectorEntity;
use crate::repositories::{RepositoryError, Result};

#[derive(Debug)]
pub struct ProjectorsMemoryStore {
    records: RwLock<HashMap<String, ProjectorEntity>>,
}

impl Default for ProjectorsMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl ProjectorsMemoryStore {
    pub fn add(&self, projector: ProjectorEntity) -> Result<()> {
        if self.records.read().unwrap().contains_key(projector.id()) {
            Err(RepositoryError::Duplicate(projector.id().into()))
        } else {
            self.records
                .write()
                .unwrap()
                .insert(projector.id().to_string(), projector);
            Ok(())
        }
    }

    pub fn all(&self) -> Vec<ProjectorEntity> {
        let projectors: Vec<ProjectorEntity> = self
            .records
            .read()
            .unwrap()
            .values()
            .map(|p| p.to_owned())
            .collect();
        projectors
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        debug!("deleting projector with id {}", id);
        if self.records.write().unwrap().remove(id).is_some() {
            Ok(())
        } else {
            Err(RepositoryError::NotFound(id.into()))
        }
    }

    pub fn exists(&self, id: &str) -> Result<bool> {
        Ok(self.records.read().unwrap().contains_key(id))
    }

    pub fn get(&self, id: &str) -> Result<ProjectorEntity> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(RepositoryError::NotFound(id.into()))
        }
    }
}
