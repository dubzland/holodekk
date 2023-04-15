mod subroutine;
pub use subroutine::*;

pub mod subroutine_definition;
pub use subroutine_definition::*;

pub mod manifest;
pub use manifest::SubroutineManifest;

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::config::{
        fixtures::{mock_config, MockConfig},
        HolodekkConfig, ProjectorConfig,
    };
    use crate::core::repositories::RepositoryId;

    use super::*;

    #[fixture]
    pub(crate) fn subroutine(
        mock_config: MockConfig,
        subroutine_definition: SubroutineDefinition,
    ) -> Subroutine {
        Subroutine::new(
            mock_config.fleet(),
            mock_config.namespace(),
            "/tmp/holodekk/projector/local/subroutines/test/sub",
            &subroutine_definition.id(),
        )
    }

    #[fixture]
    pub(crate) fn subroutine_definition() -> SubroutineDefinition {
        SubroutineDefinition::new(
            "test/sub",
            "/tmp/holodekk/subroutines/test/sub",
            SubroutineKind::Ruby,
        )
    }
}
