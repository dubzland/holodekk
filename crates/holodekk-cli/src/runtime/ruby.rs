use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;

use colored::*;

use tar::Builder as TarBuilder;

use holodekk_engine::{docker, Build, ImageKind};
// use holodekk::projector::server::ProjectorServer;
use holodekk::subroutine::{ContainerManifest, SubroutineManifest};
use holodekk::Holodekk;
use uhura::api::client::UhuraClient;

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
    async fn project(&self) -> holodekk::Result<()> {
        // let manifest = self.subroutine();

        // Start a projector
        self.holodekk.projector_for_namespace("local")?;
        let projector = self.holodekk.projector_for_namespace("local")?;
        let uhura_listener = &projector.uhura_listener;
        let client = if uhura_listener.port().is_some() {
            UhuraClient::connect_tcp(
                uhura_listener.port().unwrap().to_owned(),
                uhura_listener.address().unwrap().to_owned(),
            )
            .await
            .unwrap()
        } else {
            UhuraClient::connect_uds(uhura_listener.socket().unwrap())
                .await
                .unwrap()
        };
        let response = client.core().status().await.unwrap();
        println!("Server response: {:?}", response);
        // self.holodekk.stop_projector(projector)?;

        // holodekk.stop_projector(projector)?;

        // holodekk.stop()?;

        // Create the subroutine
        // let sub = Subroutine::new(
        //     manifest.name(),
        //     &self.directory,
        //     self.file.as_path().file_stem().unwrap().to_str().unwrap(),
        // );
        // let projector = ProjectorServer::new().listen_tcp(None, None).unwrap();
        // let port = projector.port();
        // println!("Projector running on port {}.", port);

        // // Check to see if an image exists
        // print!("Checking for application image ...");
        // let engine = docker::Docker::new();
        // if engine
        //     .image_exists(ImageKind::Application, subroutine.name())
        //     .await
        //     .unwrap()
        // {
        //     println!(" ok.");
        // } else {
        //     println!(" not found.");
        //     self.build().await;
        // }

        // println!(
        //     "{} {} {}",
        //     "Launching subroutine".green(),
        //     subroutine.name().to_string().white().bold(),
        //     "on the Holodekk.".green()
        // );

        // // Start a projector
        // let projector = ProjectorServer::new().listen_tcp(None, None).unwrap();
        // let port = projector.port();
        // println!("Projector running on port {}.", port);

        // // Launch the subroutine
        // Command::new(&self.file)
        //     .current_dir(&self.directory)
        //     .arg("start")
        //     .arg("--projector-port")
        //     .arg(port.to_string())
        //     .status()
        //     .unwrap();

        // projector.stop();
        Ok(())
    }
}

fn create_archive<T: std::io::Write, P: AsRef<Path>>(context: P, target: T) -> std::io::Result<()> {
    let mut archive: TarBuilder<T> = TarBuilder::new(target);
    archive.append_dir_all("", context.as_ref())
}
