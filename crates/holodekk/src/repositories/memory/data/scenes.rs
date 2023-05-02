use std::{collections::HashMap, sync::RwLock};

use log::debug;

use crate::entities::{EntityRepositoryError, EntityRepositoryResult, SceneEntity, SceneEntityId};

#[derive(Debug)]
pub struct ScenesMemoryStore {
    records: RwLock<HashMap<SceneEntityId, SceneEntity>>,
}

impl Default for ScenesMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl ScenesMemoryStore {
    pub fn add(&self, scene: SceneEntity) -> EntityRepositoryResult<()> {
        if self.records.read().unwrap().contains_key(&scene.id) {
            Err(EntityRepositoryError::Conflict(format!(
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

    pub fn all(&self) -> Vec<SceneEntity> {
        let scenes: Vec<SceneEntity> = self
            .records
            .read()
            .unwrap()
            .values()
            .map(|p| p.to_owned())
            .collect();
        scenes
    }

    pub fn delete(&self, id: &SceneEntityId) -> EntityRepositoryResult<()> {
        debug!("deleting scene with id {}", id);
        if self.records.write().unwrap().remove(id).is_some() {
            Ok(())
        } else {
            Err(EntityRepositoryError::NotFound(id.to_owned()))
        }
    }

    pub fn exists(&self, id: &SceneEntityId) -> EntityRepositoryResult<bool> {
        Ok(self.records.read().unwrap().contains_key(id))
    }

    pub fn get(&self, id: &SceneEntityId) -> EntityRepositoryResult<SceneEntity> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(EntityRepositoryError::NotFound(id.to_owned()))
        }
    }

    pub fn update(&self, scene: SceneEntity) -> EntityRepositoryResult<SceneEntity> {
        self.add(scene.clone())?;
        Ok(scene)
    }
}
