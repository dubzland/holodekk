use std::env;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::str;

use holodekk_core::subroutine::Subroutine;

use crate::errors::HolodekkError;

pub type Result<T> = std::result::Result<T, HolodekkError>;

pub struct Builder {
}

impl Builder {
    pub fn new() -> Self {
        Self {
        }
    }

    pub async fn build(&mut self) -> Result<()> {
        // Look for .holodekk/default.*
        let mut holodekk_directory = env::current_dir()?;
        holodekk_directory.push(".holodekk");

        let mut subroutine_file = PathBuf::from(&holodekk_directory);
        subroutine_file.push("default.rb");

        // Run it to create a manifest
        if subroutine_file.exists() {
            let subroutine: Subroutine = self.read_subroutine(&holodekk_directory, &subroutine_file)?;
            // Tar and build a container out of the subroutine
            subroutine.build(&holodekk_directory).await?;

            // Run it
            Ok(())
        } else {
            Err(HolodekkError::SubroutineNotFound)
        }
    }

   fn read_subroutine(&self, directory: &PathBuf, file: &PathBuf) -> Result<Subroutine> {
        let output = ProcessCommand::new(&file)
            .current_dir(&directory)
            .arg("manifest")
            .output()?;

        if output.status.success() {
            let data = str::from_utf8(&output.stdout).unwrap();
            let subroutine: Subroutine = serde_json::from_str(&data)
                .map_err( |e| HolodekkError::Parse(e))?;
            Ok(subroutine)
        } else {
            Err(HolodekkError::CommandExecution(output))
        }
    }
}
