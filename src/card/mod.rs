//! Deluge expects a specific folder structure a the root of a card:
//!
//! ```bash
//! tree -d -L 1
//! .
//! ├── KITS
//! ├── SAMPLES
//! └── SYNTHS
//! ```

mod filesystem;
mod patch_name;

use std::str::FromStr;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub use filesystem::{FileSystem, LocalFileSystem};
pub use patch_name::PatchName;

use crate::PatchType;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum CardError {
    #[error("directory '{0}' does not exists")]
    DirectoryDoesNotExists(PathBuf),

    #[error("missing root directory '{0}'")]
    MissingRootDirectory(String),

    // Store a String instead of std::io::Error to be able to derive PartialEq.
    #[error("I/O error: {0}")]
    IoError(String),
}

fn make_io_error(error: std::io::Error) -> CardError {
    CardError::IoError(error.to_string())
}

#[derive(Debug, EnumIter)]
pub enum CardFolder {
    Kits,
    Samples,
    Synths,
}

impl CardFolder {
    pub const fn directory_name(&self) -> &'static str {
        match self {
            CardFolder::Kits => "KITS",
            CardFolder::Samples => "SAMPLES",
            CardFolder::Synths => "SYNTHS",
        }
    }
}

/// A deluge card
/// 
/// Represents the card on the file system.
/// You should normally don't have to care about the lifetime 'l, just pass you file system and the 
/// compiler should be able to deduce everything.
#[derive(Debug)]
pub struct Card<'l, FS: FileSystem> {
    root_directory: PathBuf,
    file_system: &'l FS,
}

impl<'l, FS: FileSystem> PartialEq for Card<'l, FS> {
    fn eq(&self, other: &Self) -> bool {
        self.root_directory == other.root_directory
    }
}

impl<'l, FS: FileSystem> Card<'l, FS> {
    fn check_root_directories(file_system: &'l FS, root_directory: &Path) -> Result<(), CardError> {
        let directory_names = file_system
            .get_directory_entries(root_directory)?
            .iter()
            .filter_map(|path| path.file_name().map(|file_name| file_name.to_string_lossy().to_string()))
            .collect::<BTreeSet<String>>();

        for required_directory in CardFolder::iter() {
            if !directory_names.contains(required_directory.directory_name()) {
                return Err(CardError::MissingRootDirectory(
                    required_directory.directory_name().to_owned(),
                ));
            }
        }

        Ok(())
    }

    /// Creates the card directory and the required folders.
    pub fn create(file_system: &'l FS, root_directory: &Path) -> Result<Self, CardError> {
        let root_directory = root_directory.to_path_buf();

        if !file_system.directory_exists(&root_directory) {
            return Err(CardError::DirectoryDoesNotExists(root_directory));
        }

        let card = Self {
            file_system,
            root_directory,
        };

        for required_directory in CardFolder::iter() {
            file_system.create_directory(&card.get_directory_path(required_directory))?;
        }

        Ok(card)
    }

    /// Open a card directory.
    ///
    /// The folder structure is checked and an error is returned if something wrong is found.
    pub fn open(file_system: &'l FS, root_directory: &Path) -> Result<Self, CardError> {
        let root_directory = root_directory.to_path_buf();

        if !file_system.directory_exists(&root_directory) {
            return Err(CardError::DirectoryDoesNotExists(root_directory));
        }

        Self::check_root_directories(file_system, &root_directory)?;

        Ok(Self {
            file_system,
            root_directory,
        })
    }

    /// Get one of the card's directory path
    pub fn get_directory_path(&self, folder: CardFolder) -> PathBuf {
        self.root_directory.join(folder.directory_name())
    }

    /// Gets the next standard patch name
    ///
    /// With Deluge, when you create a patch it gets a default name. For example with kits, the first default
    /// kit is "KIT000". The next one is "KIT001". Also you can have variation of the same patch composed by the original name with a letter as postfix, example: "KIT001A". For synths patch the base name is "SYNT" instead of "KIT".
    /// Those are what I call standard patch names.
    /// The other names not respecting this pattern I call them custom patch names.
    /// Those can also have a number but this is optional and they can't have a letter (I'm not sure of that).
    pub fn get_next_standard_patch_name(&self, patch_type: PatchType) -> Result<String, CardError> {
        let folder = match patch_type {
            PatchType::Kit => CardFolder::Kits,
            PatchType::Synth => CardFolder::Synths,
        };

        let mut max_number: Option<u16> = None;

        for path in &self.file_system.get_directory_entries(&self.get_directory_path(folder))? {
            if self.file_system.is_file(path)? {
                if let Some(file_name) = path.file_name().map(|name| name.to_string_lossy().to_string()) {
                    if let Ok(PatchName::Standard {
                        patch_type: _,
                        number,
                        suffix: _,
                    }) = PatchName::from_str(&file_name)
                    {
                        max_number = Some(number.max(max_number.unwrap_or(0)))
                    }
                }
            }
        }

        Ok(PatchName::Standard {
            patch_type,
            number: max_number.map(|n| n + 1).unwrap_or(0u16),
            suffix: None,
        }
        .to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use test_case::test_case;

    use crate::PatchType;

    use super::{filesystem::MockFileSystem, Card, CardError};

    #[test]
    fn test_check_root_directories_all_correct() {
        let fs = &mut MockFileSystem::default();

        fs.expect_get_directory_entries().returning(|path| {
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

        fs.expect_get_directory_entries().returning(|path| {
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

        fs.expect_get_directory_entries().returning(|path| {
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
    fn test_open_card_non_existing_directory() {
        let fs = &mut MockFileSystem::default();

        fs.expect_directory_exists().times(1).return_const(false);
        fs.expect_get_directory_entries().times(0);
        let directory_path = Path::new("I_m_not_existings_duh");

        assert_eq!(
            Err(CardError::DirectoryDoesNotExists(directory_path.to_path_buf())),
            Card::open(fs, &directory_path)
        );
    }

    #[test]
    fn test_open_card_ok() {
        let fs = &mut MockFileSystem::default();

        fs.expect_directory_exists().times(1).return_const(true);
        fs.expect_get_directory_entries().times(1).return_once(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

        assert!(Card::open(fs, &Path::new("I_m_existings")).is_ok());
    }

    fn create_valid_card(mut fs: MockFileSystem, root_directory: &'static Path) -> MockFileSystem {
        fs.expect_directory_exists().return_const(true);
        fs.expect_get_directory_entries()
            .with(mockall::predicate::eq(root_directory))
            .return_once(|path| {
                let mut paths: Vec<PathBuf> = Vec::new();

                paths.push(path.join("KITS"));
                paths.push(path.join("SAMPLES"));
                paths.push(path.join("SYNTHS"));

                Ok(paths)
            });

        fs
    }

    #[test_case("KIT000", "KIT001" ; "KIT000")]
    #[test_case("KIT", "KIT000" ; "KIT")]
    #[test_case("alariabiata", "KIT000" ; "not default kit")]
    #[test_case("KIT000A", "KIT001" ; "KIT000A")]
    fn test_get_next_patch_name(existing_patch_name: &str, expected_patch_name: &str) {
        // let fs = &mut MockFileSystem::default();
        let root_directory = Path::new("I_exist");
        let mut fs = create_valid_card(MockFileSystem::default(), root_directory);
        let existing_patch_name_for_closure = existing_patch_name.to_string();
        fs.expect_get_directory_entries().return_once(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join(existing_patch_name_for_closure));

            Ok(paths)
        });
        fs.expect_is_file().return_once(|_path| Ok(true));

        let card = Card::open(&fs, &Path::new("I_exist")).expect("open mocked card");
        let patch_name = card.get_next_standard_patch_name(PatchType::Kit).unwrap();

        assert_eq!(expected_patch_name, patch_name);
    }

    #[test]
    fn test_get_next_patch_name_max() {
        let fs = &mut MockFileSystem::default();
        let root_directory = Path::new("I_exist");

        fs.expect_directory_exists().return_const(true);
        fs.expect_get_directory_entries()
            .with(mockall::predicate::eq(root_directory))
            .return_once(|path| {
                let mut paths: Vec<PathBuf> = Vec::new();

                paths.push(path.join("KITS"));
                paths.push(path.join("SAMPLES"));
                paths.push(path.join("SYNTHS"));

                Ok(paths)
            });

        fs.expect_get_directory_entries().return_once(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KIT003"));
            paths.push(path.join("KIT007"));
            paths.push(path.join("KIT001"));

            Ok(paths)
        });
        fs.expect_is_file().return_const::<Result<bool, CardError>>(Ok(true));

        let card = Card::open(fs, &Path::new("I_exist")).expect("open mocked card");
        let patch_name = card.get_next_standard_patch_name(PatchType::Kit).unwrap();

        assert_eq!("KIT008", patch_name);
    }
}
