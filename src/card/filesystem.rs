use std::path::{Path, PathBuf};

use super::CardError;

#[cfg(test)]
use mockall::{automock, predicate::*};

fn make_io_error(error: std::io::Error) -> CardError {
    CardError::IoError(error.to_string())
}

/// This trait exists to make unit testing possible.
#[cfg_attr(test, automock)]
pub trait FileSystem {
    /// This method gives the paths of the directories present in a given directory.
    fn get_directory_entries(&self, path: &Path) -> Result<Vec<PathBuf>, CardError>;

    /// This method creates all the missing directories.
    fn create_directory(&self, path: &Path) -> Result<(), CardError>;

    /// Check if a directory exists
    fn directory_exists(&self, path: &Path) -> bool;

    /// Check if a file exists
    fn file_exists(&self, path: &Path) -> bool;

    /// Check if a path points on a file
    fn is_file(&self, path: &Path) -> Result<bool, CardError>;
}

/// The local filesystem.
///
/// A card created using this file system will read and write the local file system.
#[derive(Default)]
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn get_directory_entries(&self, path: &Path) -> Result<Vec<PathBuf>, CardError> {
        let mut results: Vec<PathBuf> = Vec::new();

        for entry in std::fs::read_dir(path).map_err(make_io_error)? {
            if let Ok(entry) = entry.map_err(make_io_error) {
                results.push(entry.path());
            }
        }

        Ok(results)
    }

    fn create_directory(&self, path: &Path) -> Result<(), CardError> {
        std::fs::create_dir_all(path).map_err(make_io_error)?;

        Ok(())
    }

    fn directory_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_dir()
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    fn is_file(&self, path: &Path) -> Result<bool, CardError> {
        Ok(path.metadata().map_err(make_io_error)?.is_file())
    }
}
