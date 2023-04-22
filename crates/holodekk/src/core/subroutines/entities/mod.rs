mod subroutine;
pub use subroutine::*;

// pub mod manifest;
// pub use manifest::SubroutineManifest;

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::core::projectors::entities::{fixtures::projector, ProjectorEntity};
    use crate::core::subroutine_definitions::entities::{
        fixtures::subroutine_definition, SubroutineDefinitionEntity,
    };

    use super::*;

    #[fixture]
    pub(crate) fn subroutine(
        projector: ProjectorEntity,
        subroutine_definition: SubroutineDefinitionEntity,
    ) -> SubroutineEntity {
        SubroutineEntity::build(&projector, &subroutine_definition)
    }
}
