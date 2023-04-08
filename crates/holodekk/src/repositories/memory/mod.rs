mod subroutines;
pub use subroutines::*;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sha2::{Digest, Sha256};

use crate::entities::Subroutine;

use super::{Error, Result};

pub trait MemoryDatabaseKey {
    fn db_key(&self) -> String;
}

pub fn subroutine_key(fleet: &str, namespace: &str, name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fleet);
    hasher.update(namespace);
    hasher.update(name);
    format!("{:x}", hasher.finalize())
}

impl MemoryDatabaseKey for Subroutine {
    fn db_key(&self) -> String {
        subroutine_key(&self.fleet, &self.namespace, &self.name)
    }
}

#[derive(Debug)]
pub struct RecordSet<T>
where
    T: Clone + MemoryDatabaseKey,
{
    records: RwLock<HashMap<String, T>>,
}

impl<T> Default for RecordSet<T>
where
    T: Clone + MemoryDatabaseKey,
{
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl<T> RecordSet<T>
where
    T: Clone + MemoryDatabaseKey,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&self, record: T) -> Result<()> {
        if self.records.read().unwrap().contains_key(&record.db_key()) {
            Err(Error::AlreadyExists)
        } else {
            self.records
                .write()
                .unwrap()
                .insert(record.db_key(), record);
            Ok(())
        }
    }

    pub fn get(&self, key: &str) -> Result<T> {
        if let Some(record) = self.records.read().unwrap().get(key) {
            Ok(record.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }
}

#[derive(Debug)]
pub struct MemoryDatabase {
    subroutines: Arc<RecordSet<Subroutine>>,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self {
            subroutines: Arc::new(RecordSet::new()),
        }
    }
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn subroutines(&self) -> Arc<RecordSet<Subroutine>> {
        self.subroutines.clone()
    }
}

#[derive(Debug)]
pub struct MemoryRepository {
    db: Arc<MemoryDatabase>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            db: Arc::new(MemoryDatabase::new()),
        }
    }
}

impl MemoryRepository {
    pub fn new(db: Arc<MemoryDatabase>) -> Self {
        Self { db }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::*;

    use super::*;

    #[fixture]
    fn subroutine() -> Subroutine {
        Subroutine::new("test-fleet", "test-namespace", "foo", PathBuf::from("/tmp"))
    }

    #[rstest]
    #[test]
    fn can_add_subroutine(subroutine: Subroutine) {
        let db = MemoryDatabase::new();

        let result = db.subroutines().add(subroutine);
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_subroutine(subroutine: Subroutine) -> Result<()> {
        let db = MemoryDatabase::new();
        let key = subroutine_key(&subroutine.fleet, &subroutine.namespace, &subroutine.name);
        db.subroutines().add(subroutine)?;

        let subroutine = db.subroutines().get(&key)?;
        assert_eq!(subroutine.name, "foo");
        Ok(())
    }
}
