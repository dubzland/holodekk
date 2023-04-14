use std::{collections::HashMap, sync::RwLock};

use crate::core::{
    entities::{Subroutine, SubroutineInstance},
    repositories::{Error, RepositoryId, Result},
};

#[derive(Debug)]
pub struct SubroutineInstancesMemoryStore {
    records: RwLock<HashMap<String, SubroutineInstance>>,
}

impl Default for SubroutineInstancesMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl SubroutineInstancesMemoryStore {
    pub fn add(&self, instance: SubroutineInstance) -> Result<()> {
        if self.records.read().unwrap().contains_key(&instance.id()) {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(instance.id(), instance);
            Ok(())
        }
    }

    pub fn all(&self) -> Result<Vec<SubroutineInstance>> {
        let instances = self
            .records
            .read()
            .unwrap()
            .values()
            .map(|i| i.to_owned())
            .collect();
        Ok(instances)
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

    pub fn get(&self, id: &str) -> Result<SubroutineInstance> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn get_all_by_subroutine(
        &self,
        subroutine: &Subroutine,
    ) -> Result<Vec<SubroutineInstance>> {
        let records = self.records.read().unwrap();
        let instances = records
            .values()
            .filter(|s| s.subroutine_id == subroutine.id())
            .map(|s| s.to_owned())
            .collect();
        Ok(instances)
    }
}
