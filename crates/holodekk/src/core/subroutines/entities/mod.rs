mod subroutine;
pub use subroutine::*;

pub mod manifest;
pub use manifest::SubroutineManifest;

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use crate::config::{
        fixtures::{mock_config, MockConfig},
        HolodekkConfig, ProjectorConfig,
    };
    use crate::core::subroutine_definitions::entities::{
        fixtures::subroutine_definition, SubroutineDefinition,
    };

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
}
