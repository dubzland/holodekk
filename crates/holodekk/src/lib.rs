pub mod core;
// mod holodekk;
// mod projectors;
pub mod repositories;
// pub use repositories::memory::{MemoryDatabase, MemoryRepository};
// mod server;
// mod subroutine_definitions;
// mod subroutines;

// pub use self::holodekk::Holodekk;

// use std::collections::HashMap;
// use std::net::Ipv4Addr;
// use std::path::PathBuf;
// use std::sync::{Arc, RwLock};

// use clap::Parser;
// use tokio::sync::mpsc::Sender;

// use crate::core::{
//     entities::SubroutineDefinitionEntity,
//     workers::{ProjectorsRequest, SubroutinesRequest},
// };
// use crate::repositories::RepositoryKind;

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct HolodekkOptions {
//     /// Path to holodekk daemon pid file
//     #[arg(long, value_name = "file", required = true)]
//     pidfile: PathBuf,

//     /// Port to listen on
//     #[arg(long, short)]
//     port: Option<u16>,

//     /// Listen address (IP)
//     #[arg(long)]
//     address: Option<Ipv4Addr>,

//     /// Listen socket (UDS)
//     #[arg(long, conflicts_with_all = ["port", "address"])]
//     socket: Option<PathBuf>,

//     /// Repository type
//     #[arg(long, value_enum, default_value = "memory")]
//     repository: RepositoryKind,
// }

#[derive(thiserror::Error, Debug)]
pub enum HolodekkError {
    // #[error("Failed to spawn projector")]
    // ProjectorSpawn(#[from] ProjectorSpawnError),
    // #[error("Failed to terminate projector")]
    // ProjectorTermination(#[from] ProjectorTerminationError),
    // #[error("Failed to spawn subroutine")]
    // SubroutineSpawn(#[from] SubroutineSpawnError),
    // #[error("Failed to terminate subroutine")]
    // SubroutineTermination(#[from] SubroutineTerminationError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

// #[derive(Debug)]
// pub struct HolodekkServices<R>
// where
//     R: Send + Sync,
// {
//     repo: Arc<R>,
//     projectors_sender: Sender<ProjectorsRequest>,
//     subroutines_sender: Sender<SubroutinesRequest>,
//     definitions: RwLock<HashMap<String, SubroutineDefinitionEntity>>,
// }

// impl<R> HolodekkServices<R>
// where
//     R: Send + Sync + 'static,
// {
//     pub fn new(
//         repo: Arc<R>,
//         projectors_sender: Sender<ProjectorsRequest>,
//         subroutines_sender: Sender<SubroutinesRequest>,
//         definitions: RwLock<HashMap<String, SubroutineDefinitionEntity>>,
//     ) -> Self {
//         Self {
//             repo,
//             projectors_sender,
//             subroutines_sender,
//             definitions,
//         }
//     }
// }

// #[cfg(test)]
// mod fixtures {
//     use std::collections::HashMap;

//     use tokio::sync::mpsc::{channel, Receiver};

//     use crate::core::repositories::fixtures::MockRepository;

//     use super::*;

//     pub fn mock_holodekk_service(
//         repo: MockRepository,
//         definitions: HashMap<String, SubroutineDefinitionEntity>,
//     ) -> (
//         Receiver<ProjectorsRequest>,
//         Receiver<SubroutinesRequest>,
//         HolodekkServices<MockRepository>,
//     ) {
//         let (projectors_tx, projectors_rx) = channel(1);
//         let (subroutines_tx, subroutines_rx) = channel(1);

//         let services = HolodekkServices::new(
//             Arc::new(repo),
//             projectors_tx,
//             subroutines_tx,
//             RwLock::new(definitions),
//         );
//         (projectors_rx, subroutines_rx, services)
//     }
// }
