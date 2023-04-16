// use async_trait::async_trait;
// #[cfg(test)]
// use mockall::{automock, predicate::*};

// use crate::core::{
//     entities::SubroutineStatus, repositories::SubroutinesRepository, services::Result,
// };

// use super::{Status, SubroutinesService};

// #[cfg_attr(test, automock)]
// #[async_trait]
// pub trait Start {
//     async fn start(&self, name: &str) -> Result<SubroutineStatus>;
// }

// #[async_trait]
// impl<T> Start for SubroutinesService<T>
// where
//     T: SubroutinesRepository,
// {
//     async fn start(&self, name: &str) -> Result<SubroutineStatus> {
//         match self.status(name).await? {
//             SubroutineStatus::Running(pid) => Ok(SubroutineStatus::Running(pid)),
//             _ => {
//                 // TODO: Call the projector service to actually start the subroutine
//                 Ok(SubroutineStatus::Unknown)
//             }
//         }
//     }
// }
