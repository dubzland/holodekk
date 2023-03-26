// use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use async_trait::async_trait;

use colored::*;

use tar::Builder as TarBuilder;

use holodekk::engine::{docker, Build, ImageKind, Store};
use holodekk::projector::server::ProjectorServer;
use holodekk::subroutine::{ContainerManifest, SubroutineManifest};

use super::CliRuntime;

#[derive(Debug)]
pub struct RubyCliRuntime {
    directory: PathBuf,
    file: PathBuf,
}

impl RubyCliRuntime {
    pub(crate) fn new(directory: &PathBuf, file: &PathBuf) -> Self {
        Self {
            directory: directory.to_owned(),
            file: file.to_owned(),
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
        let engine = docker::Docker::new();
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
    async fn run(&self) {
        let subroutine = self.subroutine();

        // Check to see if an image exists
        print!("Checking for application image ...");
        let engine = docker::Docker::new();
        if engine
            .image_exists(ImageKind::Application, subroutine.name())
            .await
            .unwrap()
        {
            println!(" ok.");
        } else {
            println!(" not found.");
            self.build().await;
        }

        println!(
            "{} {} {}",
            "Launching subroutine".green(),
            subroutine.name().to_string().white().bold(),
            "on the Holodekk.".green()
        );

        // Start a projector
        let projector = ProjectorServer::new().listen_tcp(None, None).unwrap();
        let port = projector.port();
        println!("Projector running on port {}.", port);

        // Launch the subroutine
        Command::new(&self.file)
            .current_dir(&self.directory)
            .arg("start")
            .arg("--projector-port")
            .arg(port.to_string())
            .status()
            .unwrap();

        projector.stop();
    }
}

fn create_archive<T: std::io::Write, P: AsRef<Path>>(context: P, target: T) -> std::io::Result<()> {
    let mut archive: TarBuilder<T> = TarBuilder::new(target);
    archive.append_dir_all("", context.as_ref())
}
