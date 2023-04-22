use std::fs;
use std::path::Path;

pub fn cleanup<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if path.as_ref().exists() {
        return fs::remove_file(path);
    }
    Ok(())
}

pub fn ensure_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cleanup_succeeds_when_file_exists() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut file = temp.path().to_owned();
        file.push("temp.file");
        fs::File::create(&file)?;
        cleanup(&file)
    }

    #[test]
    fn cleanup_succeeds_when_file_does_not_exist() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut file = temp.path().to_owned();
        file.push("temp.file");
        cleanup(&file)
    }
}
