mod subroutines;
pub use subroutines::*;
mod subroutine_definitions;
pub use subroutine_definitions::*;

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::MockSubroutineDefinitionsRepository;
    use super::MockSubroutinesRepository;

    #[fixture]
    pub(crate) fn subroutines_repository() -> MockSubroutinesRepository {
        MockSubroutinesRepository::default()
    }

    #[fixture]
    pub(crate) fn subroutine_definitions_repository() -> MockSubroutineDefinitionsRepository {
        MockSubroutineDefinitionsRepository::default()
    }
}
