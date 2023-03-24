use std::path::PathBuf;
use std::process::Command;

use colored::*;

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
}

impl CliRuntime for RubyCliRuntime {
    fn build(&self) {
        Command::new(&self.file)
            .current_dir(&self.directory)
            .arg("build")
            .status()
            .unwrap();
    }
    fn manifest(&self) {}
    fn run(&self) {
        let output = Command::new(&self.file)
            .current_dir(&self.directory)
            .args(["manifest"])
            .output()
            .expect("failed to execute process");

        let subroutine: subroutine::Subroutine =
            serde_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();

        println!(
            "{} {} {}",
            "Launching subroutine".green(),
            format!("{}", subroutine.name).white().bold(),
            "on the Holodekk.".green()
        );

        // Start a projector
        let projector = ProjectorServer::new().listen_tcp(None, None).unwrap();
        let port = projector.port();
        println!("Projector running on port {}.", port);

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
