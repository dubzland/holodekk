pub mod runtime;

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use thiserror::Error;

use holodekk::entities::{SubroutineKind, SubroutineManifest};
use holodekk::repositories::memory::MemoryRepository;
use holodekk::services::{SubroutineCreateInput, SubroutinesService};
use holodekk::Holodekk;

#[async_trait]
pub trait CliRuntime: Send + Sync + 'static {
    fn holodekk(&self) -> Arc<Holodekk>;
    fn subroutine_service(&self) -> Arc<SubroutinesService<MemoryRepository>> {
        let namespace = "local";
        let repo = Arc::new(MemoryRepository::default());
        Arc::new(SubroutinesService::new(
            self.holodekk().config.clone(),
            repo.clone(),
            namespace,
        ))
    }
    fn generate_manifest(&self) -> SubroutineManifest;
    fn path(&self) -> &Path;
    async fn build(&self);
    async fn project(&self) -> holodekk::HolodekkResult<()> {
        let manifest = self.generate_manifest();

        // Start a projector
        let subroutines_service = self.subroutine_service();
        match subroutines_service.status(manifest.name()).await {
            Ok(status) => {
                if let holodekk::entities::SubroutineStatus::Running(pid) = status {
                    println!("Running: {}", pid);
                } else {
                    println!("Not running");
                    self.start_subroutine().await?;
                }
            }
            Err(holodekk::services::Error::NotFound) => {
                eprintln!("Doesn't exist");
                self.create_subroutine().await?;
                self.start_subroutine().await?;
            }
            Err(err) => {
                eprintln!("Something went wrong: {}", err);
            }
        }
        // // self.holodekk.projector_for_namespace("local")?;
        // // let projector = self.holodekk.projector_for_namespace("local")?;
        // // let client = projector.client().await?;
        // // let response = client.uhura().status().await.unwrap();
        // // println!("Server response: {:?}", response);
        // // self.holodekk.stop_projector(projector.id)?;

        Ok(())
    }

    async fn create_subroutine(&self) -> holodekk::HolodekkResult<()> {
        let manifest = self.generate_manifest();
        let input = SubroutineCreateInput {
            name: manifest.name(),
            path: self.path(),
            kind: SubroutineKind::Ruby,
        };
        let subroutines_service = self.subroutine_service();
        match subroutines_service.create(input).await {
            Ok(subroutine) => {
                println!("Subroutine created: {:?}", subroutine);
                Ok(())
            }
            Err(err) => {
                eprintln!("Unable to create subroutine: {}", err);
                Ok(())
            }
        }
    }

    async fn start_subroutine(&self) -> holodekk::HolodekkResult<()> {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CliRuntimeError {
    #[error("Invalid argument: {0}")]
    ArgumentError(String),
    #[error("Unknown Error")]
    Unknown,
}
