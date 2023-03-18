use crate::persistence::{QueryResult, SubroutineRepo};
use crate::persistence::models::Subroutine;

pub struct SqliteSubroutineRepo {
}

impl SubroutineRepo for SqliteSubroutineRepo {
    fn find_subroutine(name: &str) -> QueryResult<Subroutine> {
        Ok(Some(Subroutine::new(name)))
    }
}
