use std::path::PathBuf;

use async_trait::async_trait;
use log::trace;

use crate::core::{entities::SubroutineDefinitionEntity, enums::SubroutineKind};
use crate::HolodekkServices;

use super::{FindSubroutineDefinitions, FindSubroutineDefinitionsError};

#[async_trait]
impl<R> FindSubroutineDefinitions for HolodekkServices<R>
where
    R: Send + Sync,
{
    async fn find_subroutine_definitions<'a>(
        &self,
        name: Option<&'a str>,
        path: Option<&'a PathBuf>,
        kind: Option<SubroutineKind>,
    ) -> std::result::Result<Vec<SubroutineDefinitionEntity>, FindSubroutineDefinitionsError> {
        trace!(
            "SubroutineDefinitionsService.find({:?}, {:?}, {:?})",
            name,
            path,
            kind
        );
        let definitions = self
            .definitions
            .read()
            .unwrap()
            .values()
            .filter_map(|s| {
                if let Some(name) = name {
                    if name != s.name() {
                        return None;
                    }
                }
                if let Some(path) = path {
                    if path != s.path() {
                        return None;
                    }
                }

                if let Some(kind) = kind {
                    if kind != s.kind() {
                        return None;
                    }
                }
                Some(s.to_owned())
            })
            .collect();
        Ok(definitions)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::*;

    use crate::core::{
        entities::{fixtures::mock_subroutine_definition, SubroutineDefinitionEntity},
        repositories::fixtures::{mock_repository, MockRepository},
    };
    use crate::fixtures::mock_holodekk_service;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_existing(
        mock_repository: MockRepository,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let mut definitions = HashMap::new();
        definitions.insert(
            mock_subroutine_definition.id().to_string(),
            mock_subroutine_definition.clone(),
        );

        let (_, _, service) = mock_holodekk_service(mock_repository, definitions);

        let definitions = service
            .find_subroutine_definitions(
                Some(mock_subroutine_definition.name()),
                Some(mock_subroutine_definition.path()),
                Some(mock_subroutine_definition.kind()),
            )
            .await
            .unwrap();

        assert_eq!(definitions, vec![mock_subroutine_definition]);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_nothing_when_no_matches(
        mock_repository: MockRepository,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let mut definitions = HashMap::new();
        definitions.insert(
            mock_subroutine_definition.id().to_string(),
            mock_subroutine_definition.clone(),
        );

        let (_, _, service) = mock_holodekk_service(mock_repository, definitions);

        let definitions = service
            .find_subroutine_definitions(
                Some(&format!("{}bogus", mock_subroutine_definition.name())),
                Some(mock_subroutine_definition.path()),
                Some(mock_subroutine_definition.kind()),
            )
            .await
            .unwrap();
        assert!(definitions.is_empty());
    }
}
