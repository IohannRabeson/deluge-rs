//! Deluge expects a specific folder structure a the root of a card:
//!
//! ```bash
//! tree -d -L 1
//! .
//! ├── KITS
//! ├── SAMPLES
//! └── SYNTHS
//! ```

mod card_folder;
mod filesystem;
mod patch_name;

#[cfg(test)]
mod tests;

use std::str::FromStr;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use strum::IntoEnumIterator;

pub use card_folder::CardFolder;
pub use filesystem::{FileSystem, LocalFileSystem};
pub use patch_name::PatchName;

use crate::values::SamplePath;
use crate::PatchType;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum CardError {
    #[error("Directory '{0}' does not exists")]
    DirectoryDoesNotExists(PathBuf),

    #[error("Missing root directory '{0}'")]
    MissingRootDirectory(String),

    // Store a String instead of std::io::Error to be able to derive PartialEq.
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("The file '{0}' is not located on a Deluge card")]
    FileNotInCard(PathBuf),

    #[error("The path '{0}' is not relative")]
    PathNotRelative(PathBuf),

    #[error("No more standard name available")]
    NoMoreStandardName,

    #[error("No more postfix letter available")]
    NoMorePostfixLetter,
}

/// A deluge card
///
/// Represents the card on the file system.
/// You should normally don't have to care about the lifetime 'l, just pass your file system and the
/// compiler should be able to deduce everything for you. More precisely, the filesystem instance must live
/// at least while the card instance lives.
/// ```
/// # use std::path::Path;
/// # use deluge::{LocalFileSystem, PatchType, CardError, CardFolder};
/// if let Ok(card) = deluge::Card::open(&LocalFileSystem::default(), Path::new("your card directory")) {
///     println!("Kits directory: {:?}", card.get_directory_path(CardFolder::Kits));
///     println!("Next kit name: {}", card.get_next_standard_patch_name(PatchType::Kit)?);
/// }
/// # Ok::<(), CardError>(())
/// ```
///
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

    /// Get the root directory
    pub fn root_directory(&self) -> &Path {
        self.root_directory.as_path()
    }

    /// Create a SamplePath relative to the card root
    pub fn sample_path(&self, path: &Path) -> Result<SamplePath, CardError> {
        match path.starts_with(self.root_directory()) {
            true => Ok(SamplePath::new(
                &path
                    .strip_prefix(self.root_directory())
                    .unwrap_or_else(|e| panic!("strip prefix of '{:?}': {:?}", self.root_directory(), e))
                    .to_string_lossy()
                    .to_string(),
            )?),
            false => Err(CardError::FileNotInCard(path.to_path_buf())),
        }
    }

    /// Get the absolute path of a sample on the card
    pub fn absolute_path(&self, path: &SamplePath) -> PathBuf {
        self.root_directory.as_path().join(path.to_path())
    }

    /// Get one of the card's directory path
    pub fn get_directory_path(&self, folder: CardFolder) -> PathBuf {
        self.root_directory.join(folder.directory_name())
    }

    /// Get the next standard patch path with name and extension
    pub fn get_next_standard_patch_path(&self, patch_type: PatchType) -> Result<PathBuf, CardError> {
        let base_name = self.get_next_standard_patch_name(patch_type)?;
        let mut result = self.get_directory_path(patch_type.get_card_folder());

        result.push(base_name);
        result.set_extension("XML");

        Ok(result)
    }

    /// Gets the next standard patch name
    ///
    /// With Deluge, when you create a patch it gets a default name. For example with kits, the first default
    /// kit is "KIT000". The next one is "KIT001". Also you can have variation of the same patch composed by the original name with a letter as postfix, example: "KIT001A". For synths patch the base name is "SYNT" instead of "KIT".
    /// Those are what I call standard patch names.
    /// The other names not respecting this pattern I call them custom patch names.
    /// Those can also have a number but this is optional and they can't have a letter (I'm not sure of that).
    pub fn get_next_standard_patch_name(&self, patch_type: PatchType) -> Result<String, CardError> {
        //! I assume the maximum is 3 digits but actually Deluge has a 4 digits screen so I'm not sure.
        const MAX_STANDARD_PATCH_NUMBER: u16 = 999;
        let folder = patch_type.get_card_folder();
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

        if let Some(max_number) = max_number {
            if max_number >= MAX_STANDARD_PATCH_NUMBER {
                return Err(CardError::NoMoreStandardName);
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
