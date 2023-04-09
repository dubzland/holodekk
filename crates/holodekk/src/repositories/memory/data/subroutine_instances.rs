use std::{collections::HashMap, sync::RwLock};

use crate::entities::{Subroutine, SubroutineInstance};
use crate::repositories::{Error, Result};

use crate::repositories::memory::MemoryDatabaseKey;

#[derive(Debug)]
pub struct SubroutineInstanceMemoryStore {
    records: RwLock<HashMap<String, SubroutineInstance>>,
}

impl Default for SubroutineInstanceMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl SubroutineInstanceMemoryStore {
    pub fn add(&self, instance: SubroutineInstance) -> Result<()> {
        if self
            .records
            .read()
            .unwrap()
            .contains_key(&instance.db_key())
        {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(instance.db_key(), instance);
            Ok(())
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
            .filter(|s| s.subroutine_id == subroutine.id)
            .map(|s| s.to_owned())
            .collect();
        Ok(instances)
    }
}
