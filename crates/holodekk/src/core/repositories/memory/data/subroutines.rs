use std::{collections::HashMap, sync::RwLock};

use crate::core::repositories::{RepositoryError, RepositoryId, Result};
use crate::core::subroutines::entities::Subroutine;

#[derive(Debug)]
pub struct SubroutinesMemoryStore {
    records: RwLock<HashMap<String, Subroutine>>,
}

impl Default for SubroutinesMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl SubroutinesMemoryStore {
    pub fn add(&self, subroutine: Subroutine) -> Result<()> {
        if self.records.read().unwrap().contains_key(&subroutine.id()) {
            Err(RepositoryError::Duplicate(subroutine.id()))
        } else {
            self.records
                .write()
                .unwrap()
                .insert(subroutine.id(), subroutine);
            Ok(())
        }
    }

    pub fn all(&self) -> Vec<Subroutine> {
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

    pub fn get(&self, id: &str) -> Result<Subroutine> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(RepositoryError::NotFound(id.into()))
        }
    }

    //     pub fn get_all_by_definition(
    //         &self,
    //         subroutine: &Subroutine,
    //     ) -> Result<Vec<SubroutineInstance>> {
    //         let records = self.records.read().unwrap();
    //         let instances = records
    //             .values()
    //             .filter(|s| s.subroutine_id == subroutine.id())
    //             .map(|s| s.to_owned())
    //             .collect();
    //         Ok(instances)
    //     }
}
