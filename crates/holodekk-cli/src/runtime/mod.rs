mod ruby;

use std::env;
use std::path::PathBuf;

use super::{CliRuntime, CliRuntimeError};

pub fn detect(directory: &str, name: &str) -> Result<Box<dyn CliRuntime>, CliRuntimeError> {
    let current_dir = env::current_dir().unwrap();
    let mut holodekk_dir = PathBuf::from(current_dir);
    holodekk_dir.push(directory);

    if holodekk_dir.try_exists().unwrap() {
        let mut ruby_path = PathBuf::from(&holodekk_dir);
        ruby_path.push(format!("{}.rb", name));
        if ruby_path.try_exists().unwrap() {
            Ok(Box::new(ruby::RubyCliRuntime::new(
                &holodekk_dir,
                &ruby_path,
            )))
        } else {
            Err(CliRuntimeError::ArgumentError(format!(
                "subroutine ({}) not found",
                name
            )))
        }
    } else {
        Err(CliRuntimeError::ArgumentError(format!(
            "Holodekk directory ({}) does not exist",
            directory
        )))
    }
}
