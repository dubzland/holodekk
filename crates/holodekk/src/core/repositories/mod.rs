pub mod memory;

mod projectors;
pub use projectors::*;

mod subroutines;
pub use subroutines::*;

mod subroutine_definitions;
pub use subroutine_definitions::*;

use clap::ValueEnum;

#[derive(thiserror::Error, Clone, Copy, Debug, PartialEq)]
pub enum Error {
    #[error("General Error")]
    General,
    #[error("Entity not found")]
    NotFound,
    #[error("Record already exists")]
    AlreadyExists,
    #[error("Relation not found")]
    RelationNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum RepositoryKind {
    Memory,
}

pub trait RepositoryId {
    fn id(&self) -> String;
}

pub trait RepositoryQuery: Default + Send {
    type Entity;

    fn builder() -> Self;
    fn matches(&self, record: &Self::Entity) -> bool;
    fn build(&self) -> Self;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::MockProjectorsRepository;
    use super::MockSubroutineDefinitionsRepository;
    use super::MockSubroutinesRepository;

    #[fixture]
    pub(crate) fn projectors_repository() -> MockProjectorsRepository {
        MockProjectorsRepository::default()
    }

    #[fixture]
    pub(crate) fn subroutines_repository() -> MockSubroutinesRepository {
        MockSubroutinesRepository::default()
    }

    #[fixture]
    pub(crate) fn subroutine_definitions_repository() -> MockSubroutineDefinitionsRepository {
        MockSubroutineDefinitionsRepository::default()
    }
}
