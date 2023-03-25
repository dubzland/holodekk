use std::path::PathBuf;
use std::process::Command;

use async_trait::async_trait;

use colored::*;

use holodekk_core::engine::{docker, ImageBuilder};
use holodekk_core::subroutine;

use holodekk_projector::server::ProjectorServer;

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

    fn subroutine(&self) -> subroutine::Subroutine {
        let output = Command::new(&self.file)
            .current_dir(&self.directory)
            .args(["manifest"])
            .output()
            .expect("failed to execute process");

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
            subroutine.name.to_string().white().bold(),
            "via Docker.".cyan()
        );
        let docker = docker::Service::new();
        docker.build_application(&subroutine).await.unwrap();
        println!("{}", "Build complete.".cyan());
    }
    fn manifest(&self) {}
    async fn run(&self) {
        let subroutine = self.subroutine();

        // Check to see if an image exists
        print!("Checking for application image ...");
        if subroutine.container_image_exists().await.unwrap() {
            println!(" ok.");
        } else {
            println!(" not found.");
            self.build().await;
        }

        println!(
            "{} {} {}",
            "Launching subroutine".green(),
            subroutine.name.to_string().white().bold(),
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
