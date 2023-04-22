mod data;
pub use data::*;
mod projectors;
pub use projectors::*;
mod subroutines;
pub use subroutines::*;

use std::sync::Arc;

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

    use crate::core::subroutines::entities::fixtures::subroutine;
    use crate::repositories::Result;

    use super::*;

    #[rstest]
    #[test]
    fn can_add_subroutine(subroutine: SubroutineEntity) {
        let db = MemoryDatabase::new();

        let result = db.subroutines().add(subroutine.to_owned());
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_subroutine(subroutine: SubroutineEntity) -> Result<()> {
        let db = MemoryDatabase::new();
        // let key = subroutine_key(&subroutine.id);
        db.subroutines().add(subroutine.clone())?;

        let new_sub = db.subroutines().get(&subroutine.id())?;
        assert_eq!(new_sub.id(), subroutine.id());
        Ok(())
    }
}
