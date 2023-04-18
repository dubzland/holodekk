use async_trait::async_trait;
use log::trace;

use crate::core::repositories::RepositoryQuery;
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, repositories::SubroutineDefinitionsQuery,
    FindSubroutineDefinitions, Result, SubroutineDefinitionsFindInput,
};

use super::SubroutineDefinitionsService;

impl From<&'_ SubroutineDefinitionsFindInput<'_>> for SubroutineDefinitionsQuery {
    fn from(value: &SubroutineDefinitionsFindInput) -> Self {
        let mut query = SubroutineDefinitionsQuery::builder();
        if let Some(name) = value.name() {
            query.name_eq(name);
        }
        if let Some(path) = value.path() {
            query.path_eq(path);
        }
        if let Some(kind) = value.kind() {
            query.kind_eq(kind);
        }
        query.build()
    }
}

#[async_trait]
impl FindSubroutineDefinitions for SubroutineDefinitionsService {
    async fn find<'a>(
        &self,
        input: &'a SubroutineDefinitionsFindInput<'a>,
    ) -> Result<Vec<SubroutineDefinition>> {
        trace!("SubroutineDefinitionsService.find({:?})", input);
        let query = SubroutineDefinitionsQuery::from(input);
        let definitions = self
            .definitions
            .read()
            .unwrap()
            .values()
            // .iter()
            .filter_map(|s| {
                if query.matches(s) {
                    Some(s.to_owned())
                } else {
                    None
                }
            })
            .collect();
        Ok(definitions)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::RwLock;

    use rstest::*;

    use crate::core::subroutine_definitions::{
        entities::{fixtures::subroutine_definition, SubroutineDefinition},
        Result,
    };
    //     use crate::config::fixtures::{mock_config, MockConfig};
    //     use crate::core::projectors::{
    //         entities::{fixtures::projector, Projector},
    //         repositories::{fixtures::projectors_repository, MockProjectorsRepository},
    //         worker::{fixtures::mock_projectors_worker, MockProjectorsWorker},
    //         Result,
    //     };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_existing(subroutine_definition: SubroutineDefinition) -> Result<()> {
        let mut definitions = HashMap::new();
        definitions.insert(
            subroutine_definition.id().to_string(),
            subroutine_definition.clone(),
        );

        let service = SubroutineDefinitionsService::new(RwLock::new(definitions));
        let definitions = service
            .find(&SubroutineDefinitionsFindInput::new(
                Some(subroutine_definition.name()),
                Some(subroutine_definition.path()),
                Some(subroutine_definition.kind()),
            ))
            .await?;
        assert_eq!(definitions, vec![subroutine_definition]);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_nothing_when_no_matches(
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let mut definitions = HashMap::new();
        definitions.insert(
            subroutine_definition.id().to_string(),
            subroutine_definition.clone(),
        );

        let service = SubroutineDefinitionsService::new(RwLock::new(definitions));
        let definitions = service
            .find(&SubroutineDefinitionsFindInput::new(
                Some(&format!("{}bogus", subroutine_definition.name())),
                Some(subroutine_definition.path()),
                Some(subroutine_definition.kind()),
            ))
            .await?;
        assert!(definitions.is_empty());
        Ok(())
    }
}
