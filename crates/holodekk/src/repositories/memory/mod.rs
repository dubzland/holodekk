mod data;
pub use data::*;
mod scenes;
pub use scenes::*;
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

    // use crate::core::subroutines::entities::fixtures::subroutine;
    use crate::core::entities::{fixtures::mock_scene, SceneEntity};

    use super::*;

    #[rstest]
    #[test]
    fn can_add_scene(mock_scene: SceneEntity) {
        let db = MemoryDatabase::new();

        let result = db.scenes().add(mock_scene);
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_scene(mock_scene: SceneEntity) {
        let db = MemoryDatabase::new();
        db.scenes().add(mock_scene.clone()).unwrap();

        let new_scene = db.scenes().get(&mock_scene.id).unwrap();
        assert_eq!(new_scene.id, mock_scene.id);
    }

    // #[rstest]
    // #[test]
    // fn can_get_subroutine(subroutine: SubroutineEntity) -> Result<()> {
    //     let db = MemoryDatabase::new();
    //     // let key = subroutine_key(&subroutine.id);
    //     db.subroutines().add(subroutine.clone())?;

    //     let new_sub = db.subroutines().get(&subroutine.id())?;
    //     assert_eq!(new_sub.id(), subroutine.id());
    //     Ok(())
    // }

    // #[rstest]
    // #[test]
    // fn can_add_subroutine(subroutine: SubroutineEntity) {
    //     let db = MemoryDatabase::new();

    //     let result = db.subroutines().add(subroutine.to_owned());
    //     assert!(result.is_ok())
    // }

    // #[rstest]
    // #[test]
    // fn can_get_subroutine(subroutine: SubroutineEntity) -> Result<()> {
    //     let db = MemoryDatabase::new();
    //     // let key = subroutine_key(&subroutine.id);
    //     db.subroutines().add(subroutine.clone())?;

    //     let new_sub = db.subroutines().get(&subroutine.id())?;
    //     assert_eq!(new_sub.id(), subroutine.id());
    //     Ok(())
    // }
}
