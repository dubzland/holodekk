pub mod runtime;

use std::path::PathBuf;
// use std::path::{Path, PathBuf};
// use std::process::Command;

use thiserror::Error;

// use holodekk::core::{
//     entities::{SubroutineKind, SubroutineManifest, SubroutineStatus},
//     repositories::{memory::MemoryRepository, ProjectorRepository, SubroutineRepository},
//     services::{
//         self,
//         subroutines::{Create, Status, SubroutineCreateInput, SubroutinesService},
//     },
// };

pub struct CliRuntime {
    _directory: PathBuf,
    _file: PathBuf,
}

impl CliRuntime {
    pub fn new<P>(directory: P, file: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            _directory: directory.into(),
            _file: file.into(),
        }
    }
    // pub fn generate_manifest(&self) -> SubroutineManifest {
    //     let output = Command::new(&self.file)
    //         .current_dir(&self.directory)
    //         .args(["manifest"])
    //         .output()
    //         .expect("failed to execute process");

    //     serde_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    // }

    // pub fn subroutine_service<T>(&self) -> Arc<SubroutinesService<T>>
    // where
    //     T: ProjectorRepository + SubroutineRepository,
    // {
    //     let namespace = "local";
    //     let repo = Arc::new(MemoryRepository::default());
    //     Arc::new(SubroutinesService::new(
    //         self.config.clone(),
    //         repo,
    //         namespace,
    //     ))
    // }

    // pub fn path(&self) -> &Path {
    //     &self.directory
    // }

    // async fn project(&self) -> std::result::Result<(), std::io::Error> {
    //     let manifest = self.generate_manifest();

    //     // Start a projector
    //     let subroutines_service = self.subroutine_service();
    //     match subroutines_service.status(manifest.name()).await {
    //         Ok(status) => {
    //             if let SubroutineStatus::Running(pid) = status {
    //                 println!("Running: {}", pid);
    //             } else {
    //                 println!("Not running");
    //                 self.start_subroutine().await?;
    //             }
    //         }
    //         Err(services::Error::NotFound) => {
    //             eprintln!("Doesn't exist");
    //             self.create_subroutine().await?;
    //             self.start_subroutine().await?;
    //         }
    //         Err(err) => {
    //             eprintln!("Something went wrong: {}", err);
    //         }
    //     }

    //     Ok(())
    // }

    // async fn create_subroutine(&self) -> holodekkd::HolodekkResult<()> {
    //     let manifest = self.generate_manifest();
    //     let input = SubroutineCreateInput {
    //         name: manifest.name().into(),
    //         path: self.path().into(),
    //         kind: SubroutineKind::Ruby,
    //     };
    //     let subroutines_service = self.subroutine_service();
    //     match subroutines_service.create(input).await {
    //         Ok(subroutine) => {
    //             println!("Subroutine created: {:?}", subroutine);
    //             Ok(())
    //         }
    //         Err(err) => {
    //             eprintln!("Unable to create subroutine: {}", err);
    //             Ok(())
    //         }
    //     }
    // }

    // async fn start_subroutine(&self) -> holodekkd::HolodekkResult<()> {
    //     Ok(())
    // }
}

#[derive(Debug, Error)]
pub enum CliRuntimeError {
    #[error("Invalid argument: {0}")]
    ArgumentError(String),
    #[error("Unknown Error")]
    Unknown,
}
