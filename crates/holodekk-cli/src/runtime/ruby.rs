// use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;
use colored::*;
use log::info;
use tar::Builder as TarBuilder;

use holodekk::config::HolodekkConfig;
use holodekk::core::entities::{ContainerManifest, SubroutineManifest};
use holodekk::core::repositories::memory::MemoryRepository;
use holodekk::core::services::projectors::{
    ProjectorStartInput, ProjectorStopInput, ProjectorsService, Start, Stop,
};
use holodekk::engines::{docker, Build, ImageKind};
use holodekkd::{
    holodekk::{Holodekk, HolodekkResult},
    server::HolodekkServer,
};

use super::CliRuntime;

#[derive(Debug)]
pub struct RubyCliRuntime {
    holodekk: Arc<Holodekk>,
    directory: PathBuf,
    file: PathBuf,
}

impl RubyCliRuntime {
    pub(crate) fn new(holodekk: Arc<Holodekk>, directory: &PathBuf, file: &PathBuf) -> Self {
        Self {
            holodekk,
            directory: directory.to_owned(),
            file: file.to_owned(),
        }
    }
}

#[async_trait]
impl CliRuntime for RubyCliRuntime {
    fn path(&self) -> &Path {
        &self.directory
    }
    fn holodekk(&self) -> Arc<Holodekk> {
        self.holodekk.clone()
    }
    fn generate_manifest(&self) -> SubroutineManifest {
        let output = Command::new(&self.file)
            .current_dir(&self.directory)
            .args(["manifest"])
            .output()
            .expect("failed to execute process");

        serde_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    }

    async fn build(&self) {
        let manifest = self.generate_manifest();
        println!(
            "{} {} {}",
            "Building application for".cyan(),
            manifest.name().to_string().white().bold(),
            "via Docker.".cyan()
        );
        let engine = docker::Docker::connect_local();
        let mut bytes = Vec::default();
        match manifest.container() {
            ContainerManifest::FromDockerContext { context, .. } => {
                let path = PathBuf::from(context.as_str());
                create_archive(path, &mut bytes).unwrap();
            }
        }
        engine
            .build(
                ImageKind::Application,
                manifest.name(),
                "latest",
                bytes,
                None,
            )
            .await
            .unwrap();
        println!("{}", "Build complete.".cyan());
    }
    async fn project(&self) -> HolodekkResult<()> {
        // let manifest = self.generate_manifest();

        let config = Arc::new(HolodekkConfig {
            fleet: "local".into(),
            root_path: "/home/jdubz/.holodekk/local/".into(),
            bin_path: "/home/jdubz/code/gitlab/holodekk/holodekk/target/debug".into(),
        });

        env_logger::init();

        // Start the Holodekk server
        let repo = Arc::new(MemoryRepository::default());
        let holodekk = HolodekkServer::start(config.clone(), repo.clone());
        let projector_service =
            ProjectorsService::new(config.clone(), repo.clone(), holodekk.manager_tx());

        // Start a projector
        let start = ProjectorStartInput {
            namespace: "local".into(),
        };
        let projector = projector_service.start(start).await.unwrap();

        info!("projector spawned");

        let stop = ProjectorStopInput {
            namespace: projector.namespace,
        };
        projector_service.stop(stop).await.unwrap();
        info!("projector shutdown");
        // let namespace = "local";
        // let subroutines_service = Arc::new(SubroutinesService::new(repo.clone(), fleet, namespace));
        // match subroutines_service.status(manifest.name()).await {
        //     Ok(status) => {
        //         if let holodekk::entities::SubroutineStatus::Running(pid) = status {
        //             println!("Running: {}", pid);
        //         } else {
        //             println!("Not running");
        //         }
        //     }
        //     Err(holodekk::services::Error::NotFound) => {
        //         eprintln!("Doesn't exist");
        //     }
        //     Err(err) => {
        //         eprintln!("Something went wrong: {}", err);
        //     }
        // }
        // self.holodekk.projector_for_namespace("local")?;
        // let projector = self.holodekk.projector_for_namespace("local")?;
        // let client = projector.client().await?;
        // let response = client.uhura().status().await.unwrap();
        // println!("Server response: {:?}", response);
        // self.holodekk.stop_projector(projector.id)?;

        Ok(())
    }
}

fn create_archive<T: std::io::Write, P: AsRef<Path>>(context: P, target: T) -> std::io::Result<()> {
    let mut archive: TarBuilder<T> = TarBuilder::new(target);
    archive.append_dir_all("", context.as_ref())
}
