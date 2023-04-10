mod data;
pub use data::*;
mod subroutines;
pub use subroutines::*;

use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::entities::{Subroutine, SubroutineInstance};

pub trait MemoryDatabaseKey {
    fn db_key(&self) -> String;
}

impl MemoryDatabaseKey for Subroutine {
    fn db_key(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.name);
        format!("{:x}", hasher.finalize())
    }
}

impl MemoryDatabaseKey for SubroutineInstance {
    fn db_key(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.fleet);
        hasher.update(&self.namespace);
        hasher.update(&self.subroutine_id);
        format!("{:x}", hasher.finalize())
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
    use rstest::*;

    use crate::entities::subroutine::fixtures::subroutine;
    use crate::repositories::Result;

    use super::*;

    #[rstest]
    #[test]
    fn can_add_subroutine(subroutine: Subroutine) {
        let db = MemoryDatabase::new();

        let result = db.subroutines().add(subroutine.to_owned());
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_subroutine(subroutine: Subroutine) -> Result<()> {
        let db = MemoryDatabase::new();
        // let key = subroutine_key(&subroutine.id);
        db.subroutines().add(subroutine.clone())?;

        let new_sub = db.subroutines().get(&subroutine.db_key())?;
        assert_eq!(new_sub.path, subroutine.path);
        Ok(())
    }

    #[rstest]
    #[test]
    fn can_get_subroutine_by_name(subroutine: Subroutine) -> Result<()> {
        let db = MemoryDatabase::new();
        db.subroutines().add(subroutine.to_owned())?;

        let new_sub = db.subroutines().get_by_name(&subroutine.name)?;
        assert_eq!(new_sub.path, subroutine.path);
        Ok(())
    }
}
