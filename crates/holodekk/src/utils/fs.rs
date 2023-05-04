//! Generic filesystem utilities

use std::fs;
use std::path::Path;

/// Removes the given file from the filesystem.
///
/// Checks for existence first, so safe to call on any file.  Useful for ensuring old socket files
/// aren't standing in the way.
///
/// # Errors
///
/// This method uses [`try_exists()`][`std::path::Path::try_exists()`] and
/// [`remove_file()`][`std::fs::remove_file()`], so it will return errors if the file is
/// inaccessible due to permissions.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// use holodekk::utils::fs::remove_file;
///
/// let path = PathBuf::from("/tmp/file.txt");
/// remove_file(&path).unwrap();
/// ```
pub fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if path.as_ref().try_exists()? {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

/// Recursively removes the given directory from the filesystem.
///
/// Checks for existence first, so safe to call on any directory.
///
/// # Errors
///
/// Typically permissions related errors generated by [`try_exists()`][`std::path::Path::try_exists()`].
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// use holodekk::utils::fs::remove_directory;
///
/// let path = PathBuf::from("/tmp/dir");
/// remove_directory(&path).unwrap();
/// ```
pub fn remove_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if path.as_ref().try_exists()? {
        fs::remove_dir_all(path)
    } else {
        Ok(())
    }
}

/// Ensures a given directory exists safely, creating it if not present.
///
/// # Errors
///
/// Usually related to permissions issues with parent directories (see
/// [`try_exists()`][`std::path::Path::try_exists()`]).
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// use holodekk::utils::fs::ensure_directory;
///
/// let path = PathBuf::from("/tmp/dir");
/// ensure_directory(&path).unwrap();
/// ```
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
    fn remove_file_succeeds_when_file_does_not_exist() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut file = temp.path().to_owned();
        file.push("temp.file");
        remove_file(&file)
    }

    #[test]
    fn remove_file_succeeds_when_file_exists() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut file = temp.path().to_owned();
        file.push("temp.file");
        fs::File::create(&file)?;
        remove_file(&file)
    }

    #[test]
    fn remove_file_actually_removes_the_file() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut file = temp.path().to_owned();
        file.push("temp.file");
        fs::File::create(&file)?;
        remove_file(&file)?;
        assert!(!file.try_exists()?);
        Ok(())
    }

    #[test]
    fn remove_directory_succeeds_when_directory_does_not_exist() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut dir = temp.path().to_owned();
        dir.push("dir");
        remove_directory(&dir)
    }

    #[test]
    fn remove_directory_succeeds_when_directory_exists() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut dir = temp.path().to_owned();
        dir.push("dir");
        fs::create_dir_all(&dir)
    }

    #[test]
    fn remove_directory_actually_removes_the_directory() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut dir = temp.path().to_owned();
        dir.push("dir");
        fs::create_dir_all(&dir)?;
        remove_directory(&dir)?;
        assert!(!dir.try_exists()?);
        Ok(())
    }

    #[test]
    fn ensure_directory_succeeds_when_directory_does_not_exist() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut dir = temp.path().to_owned();
        dir.push("dir");
        ensure_directory(&dir)
    }

    #[test]
    fn ensure_directory_succeeds_when_directory_exists() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut dir = temp.path().to_owned();
        dir.push("dir");
        fs::create_dir_all(&dir)
    }

    #[test]
    fn ensure_directory_creates_the_directory_when_it_does_not_exist() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let mut dir = temp.path().to_owned();
        dir.push("dir");
        ensure_directory(&dir)?;
        assert!(dir.try_exists()?);
        Ok(())
    }
}
