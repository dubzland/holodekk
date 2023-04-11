use std::{collections::HashMap, sync::RwLock};

use crate::core::{
    entities::Subroutine,
    repositories::{memory::MemoryDatabaseKey, Error, Result},
};

#[derive(Debug)]
pub struct SubroutineMemoryStore {
    records: RwLock<HashMap<String, Subroutine>>,
}

impl Default for SubroutineMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl SubroutineMemoryStore {
    pub fn add(&self, subroutine: Subroutine) -> Result<()> {
        if self
            .records
            .read()
            .unwrap()
            .contains_key(&subroutine.db_key())
        {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(subroutine.db_key(), subroutine);
            Ok(())
        }
    }

    pub fn get(&self, id: &str) -> Result<Subroutine> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<Subroutine> {
        let records = self.records.read().unwrap();
        if let Some(subroutine) = records.values().find(|s| s.name == name) {
            Ok(subroutine.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }
}
