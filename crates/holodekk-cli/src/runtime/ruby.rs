use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;

use colored::*;

use tar::Builder as TarBuilder;

use holodekk::engines::{docker, Build, ImageKind};
use holodekk::entities::{ContainerManifest, SubroutineManifest};
use holodekk::{Holodekk, HolodekkResult};
// use uhura::api::client::UhuraClient;

use super::CliRuntime;

#[derive(Debug)]
pub struct RubyCliRuntime {
    directory: PathBuf,
    file: PathBuf,
    holodekk: Arc<Holodekk>,
}

impl RubyCliRuntime {
    pub(crate) fn new(holodekk: Arc<Holodekk>, directory: &PathBuf, file: &PathBuf) -> Self {
        Self {
            directory: directory.to_owned(),
            file: file.to_owned(),
            holodekk,
        }
    }

    fn subroutine(&self) -> SubroutineManifest {
        let output = Command::new(&self.file)
            .current_dir(&self.directory)
            .args(["manifest"])
            .output()
            .expect("failed to execute process");

        //         io::stderr().write_all(&output.stdout).unwrap();
        //         io::stderr().write_all(&output.stderr).unwrap();

        serde_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    }
}

#[async_trait]
impl CliRuntime for RubyCliRuntime {
    async fn build(&self) {
        let subroutine = self.subroutine();
        println!(
            "{} {} {}",
            "Building application for".cyan(),
            subroutine.name().to_string().white().bold(),
            "via Docker.".cyan()
        );
        let engine = docker::Docker::connect_local();
        let mut bytes = Vec::default();
        match subroutine.container() {
            ContainerManifest::FromDockerContext { context, .. } => {
                let path = PathBuf::from(context.as_str());
                create_archive(path, &mut bytes).unwrap();
            }
        }
        engine
            .build(
                ImageKind::Application,
                subroutine.name(),
                "latest",
                bytes,
                None,
            )
            .await
            .unwrap();
        println!("{}", "Build complete.".cyan());
    }
    fn manifest(&self) {}
    async fn project(&self) -> HolodekkResult<()> {
        // let manifest = self.subroutine();

        // Start a projector
        self.holodekk.projector_for_namespace("local")?;
        let projector = self.holodekk.projector_for_namespace("local")?;
        let client = projector.client().await?;
        let response = client.uhura().status().await.unwrap();
        println!("Server response: {:?}", response);
        self.holodekk.stop_projector(projector.id)?;

        Ok(())
    }
}

fn create_archive<T: std::io::Write, P: AsRef<Path>>(context: P, target: T) -> std::io::Result<()> {
    let mut archive: TarBuilder<T> = TarBuilder::new(target);
    archive.append_dir_all("", context.as_ref())
}
