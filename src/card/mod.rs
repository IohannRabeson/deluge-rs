//! Deluge expects a specific folder structure:
//!
//! tree -d -L 1
//! .
//! ├── KITS
//! ├── SAMPLES
//! └── SYNTHS
//!

mod filesystem;

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub use filesystem::{FileSystem, LocalFileSystem};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum CardError {
    #[error("directory '{0}' does not exists")]
    DirectoryDoesNotExists(PathBuf),

    #[error("missing root directory '{0}'")]
    MissingRootDirectory(String),

    #[error("I/O error: {0}")]
    IoError(String),
}

/// A deluge card
#[derive(PartialEq, Debug)]
pub struct Card {
    root_directory: PathBuf,
}

impl Card {
    const REQUIRED_DIRECTORIES: [&'static str; 3] = ["KITS", "SAMPLES", "SYNTHS"];

    fn check_root_directories<FS: FileSystem>(file_system: &FS, root_directory: &Path) -> Result<(), CardError> {
        let directory_names = file_system
            .get_directories(root_directory)?
            .iter()
            .filter_map(|path| path.file_name().map(|file_name| file_name.to_string_lossy().to_string()))
            .collect::<BTreeSet<String>>();

        for required_directory in Self::REQUIRED_DIRECTORIES {
            if !directory_names.contains(required_directory) {
                return Err(CardError::MissingRootDirectory(required_directory.to_owned()));
            }
        }

        Ok(())
    }

    /// Creates the card directory and the required folders.
    pub fn create<FS: FileSystem>(file_system: &FS, root_directory: &Path) -> Result<Card, CardError> {
        let root_directory = root_directory.to_path_buf();

        if !file_system.directory_exists(&root_directory) {
            return Err(CardError::DirectoryDoesNotExists(root_directory));
        }

        for required_directory in Self::REQUIRED_DIRECTORIES {
            file_system.create_directory(&root_directory.join(required_directory))?;
        }

        Ok(Card { root_directory })
    }

    /// Open a card directory.
    ///
    /// The folder structure is checked and relevant are returned if something wrong is found.
    pub fn open<FS: FileSystem>(file_system: &FS, root_directory: &Path) -> Result<Card, CardError> {
        let root_directory = root_directory.to_path_buf();

        if !file_system.directory_exists(&root_directory) {
            return Err(CardError::DirectoryDoesNotExists(root_directory));
        }

        Self::check_root_directories(file_system, &root_directory)?;

        Ok(Card { root_directory })
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{filesystem::MockFileSystem, Card, CardError};

    #[test]
    fn test_check_root_directories_all_correct() {
        let fs = &mut MockFileSystem::default();

        fs.expect_get_directories().returning(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

        assert_eq!(Ok(()), Card::check_root_directories(fs, &Path::new("big pullayo")));
    }

    #[test]
    fn test_check_root_directories_first_missing() {
        let fs = &mut MockFileSystem::default();

        fs.expect_get_directories().returning(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("PLITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

        assert_eq!(
            Err(CardError::MissingRootDirectory("KITS".into())),
            Card::check_root_directories(fs, &Path::new("big pullayo"))
        );
    }

    #[test]
    fn test_check_root_directories_last_missing() {
        let fs = &mut MockFileSystem::default();

        fs.expect_get_directories().returning(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("FFYNYNTHS"));

            Ok(paths)
        });

        assert_eq!(
            Err(CardError::MissingRootDirectory("SYNTHS".into())),
            Card::check_root_directories(fs, &Path::new("big pullayo"))
        );
    }

    #[test]
    fn test_open_card_ok() {
        let fs = &mut MockFileSystem::default();

        fs.expect_directory_exists().times(1).return_const(true);
        fs.expect_get_directories().times(1).return_once(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

        assert!(Card::open(fs, &Path::new("I_m_existings")).is_ok());
    }

    #[test]
    fn test_open_card_non_existing_directory() {
        let fs = &mut MockFileSystem::default();

        fs.expect_directory_exists().times(1).return_const(false);
        fs.expect_get_directories().times(0);
        let directory_path = Path::new("I_m_not_existings_duh");

        assert_eq!(
            Err(CardError::DirectoryDoesNotExists(directory_path.to_path_buf())),
            Card::open(fs, &directory_path)
        );
    }
}