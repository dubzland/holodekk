pub mod models;
pub mod sqlite;

pub type QueryResult<T> = Result<Option<T>, std::io::Error>;

pub trait SubroutineRepo {
    fn find_subroutine(name: &str) -> QueryResult<models::Subroutine>;
}
