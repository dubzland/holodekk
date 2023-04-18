use async_trait::async_trait;

use crate::core::services::{Error, Result};
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, GetSubroutineDefinition, SubroutineDefinitionsGetInput,
};

use super::SubroutineDefinitionsService;

#[async_trait]
impl GetSubroutineDefinition for SubroutineDefinitionsService {
    async fn get<'a>(
        &self,
        input: &'a SubroutineDefinitionsGetInput,
    ) -> Result<SubroutineDefinition> {
        let definitions = self.definitions.read().unwrap();
        if let Some(definition) = definitions.get(input.id()) {
            Ok(definition.to_owned())
        } else {
            Err(Error::NotFound)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use rstest::*;

//     use crate::config::fixtures::{mock_config, MockConfig};
//     use crate::core::projectors::{
//         entities::{fixtures::projector, Projector},
//         repositories::{fixtures::projectors_repository, MockProjectorsRepository},
//         worker::{fixtures::mock_projectors_worker, MockProjectorsWorker},
//     };
//     use crate::core::services::{Error, Result};

//     use super::*;

//     #[rstest]
//     #[tokio::test]
//     async fn returns_projector_for_existing_projector(
//         mock_config: MockConfig,
//         mock_projectors_worker: MockProjectorsWorker,
//         mut projectors_repository: MockProjectorsRepository,
//         projector: Projector,
//     ) -> Result<()> {
//         let id = projector.id().to_string();

//         projectors_repository
//             .expect_projectors_get()
//             .withf(move |i| i == id)
//             .return_const(Ok(projector.clone()));

//         let service = ProjectorsService::new(
//             Arc::new(mock_config),
//             Arc::new(projectors_repository),
//             mock_projectors_worker,
//         );

//         assert_eq!(
//             service
//                 .get(&ProjectorsGetInput::new(projector.id()))
//                 .await?,
//             projector
//         );
//         Ok(())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn returns_error_for_nonexisting_projector(
//         mock_config: MockConfig,
//         mock_projectors_worker: MockProjectorsWorker,
//         mut projectors_repository: MockProjectorsRepository,
//     ) -> Result<()> {
//         projectors_repository
//             .expect_projectors_get()
//             .return_const(Err(crate::core::repositories::Error::NotFound));

//         let service = ProjectorsService::new(
//             Arc::new(mock_config),
//             Arc::new(projectors_repository),
//             mock_projectors_worker,
//         );

//         assert!(matches!(
//             service
//                 .get(&ProjectorsGetInput::new("nonexistent"))
//                 .await
//                 .unwrap_err(),
//             Error::NotFound
//         ));
//         Ok(())
//     }
// }
