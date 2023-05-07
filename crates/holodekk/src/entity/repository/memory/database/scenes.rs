use std::{collections::HashMap, sync::RwLock};

use log::debug;

use crate::core::scene::{entity::Id, Entity};
use crate::entity::repository::{Error, Result};

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
    pub fn add(&self, scene: Entity) -> Result<()> {
        if self.records.read().unwrap().contains_key(&scene.id) {
            Err(Error::Conflict(format!(
                "Scene already exists with id {}",
                scene.id
            )))
        } else {
            self.records
                .write()
                .unwrap()
                .insert(scene.id.clone(), scene);
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

    pub fn delete(&self, id: &Id) -> Result<()> {
        debug!("deleting scene with id {}", id);
        if self.records.write().unwrap().remove(id).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound(id.clone()))
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn exists(&self, id: &Id) -> Result<bool> {
        Ok(self.records.read().unwrap().contains_key(id))
    }

    pub fn get(&self, id: &Id) -> Result<Entity> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.clone())
        } else {
            Err(Error::NotFound(id.clone()))
        }
    }

    pub fn update(&self, scene: Entity) -> Result<Entity> {
        self.add(scene.clone())?;
        Ok(scene)
    }
}
