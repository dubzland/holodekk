use std::{collections::HashMap, sync::RwLock};

use crate::core::repositories::{Error, Result};
use crate::core::subroutine_definitions::entities::SubroutineDefinition;

#[derive(Debug)]
pub struct SubroutineDefinitionsMemoryStore {
    records: RwLock<HashMap<String, SubroutineDefinition>>,
}

impl Default for SubroutineDefinitionsMemoryStore {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl SubroutineDefinitionsMemoryStore {
    pub fn add(&self, definition: SubroutineDefinition) -> Result<()> {
        if self.records.read().unwrap().contains_key(definition.id()) {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(definition.id().into(), definition);
            Ok(())
        }
    }

    pub fn all(&self) -> Result<Vec<SubroutineDefinition>> {
        let definitions = self
            .records
            .read()
            .unwrap()
            .values()
            .map(|i| i.to_owned())
            .collect();
        Ok(definitions)
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

    pub fn get(&self, id: &str) -> Result<SubroutineDefinition> {
        if let Some(record) = self.records.read().unwrap().get(id) {
            Ok(record.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }
}
