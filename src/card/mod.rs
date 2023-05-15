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

use core::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
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

/// An error related to a Deluge card.
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum CardError {
    /// A directory does not exist.
    #[error("Directory '{0}' does not exists")]
    DirectoryDoesNotExists(PathBuf),

    /// A directory already exists.
    #[error("Directory '{0}' already exists")]
    DirectoryAlreadyExists(PathBuf),

    /// One of the root directory is missing.
    #[error("Missing root directory '{0}'")]
    MissingRootDirectory(String),

    /// I/O error.
    /// Stores a String instead of std::io::Error to be able to derive PartialEq.
    #[error("I/O error: {0}")]
    IoError(String),

    /// The path designate a file not located on the card.
    #[error("The file '{0}' is not located on a Deluge card")]
    FileNotInCard(PathBuf),

    /// The path is not relative.
    #[error("The path '{0}' is not relative")]
    PathNotRelative(PathBuf),

    /// There is no more standard name available.
    #[error("No more standard name available")]
    NoMoreStandardName,

    /// THere is no more postfix letter available.
    #[error("No more postfix letter available")]
    NoMorePostfixLetter,
}

/// A deluge card
///
/// Represents the card on the file system.
/// ```
/// # use std::path::Path;
/// # use deluge::{LocalFileSystem, PatchType, CardError, CardFolder};
/// if let Ok(card) = deluge::Card::open(LocalFileSystem::default(), Path::new("your card directory")) {
///     println!("Kits directory: {:?}", card.get_directory_path(CardFolder::Kits));
///     println!("Next kit name: {}", card.get_next_standard_patch_name(PatchType::Kit)?);
/// }
/// # Ok::<(), CardError>(())
/// ```
///
/// Generic parameter FS allows to specify the filesystem to use, this is useful for unit testing where you do not want to
/// query the real filesystem.  
///
/// Notice Card does implement Clone but the file system is never duplicated.
///
pub struct Card<FS: FileSystem> {
    root_directory: PathBuf,
    file_system: Arc<FS>,
}

impl<FS: FileSystem> Clone for Card<FS> {
    fn clone(&self) -> Self {
        Self {
            root_directory: self.root_directory.clone(),
            file_system: self.file_system.clone(),
        }
    }
}

impl<FS: FileSystem> Debug for Card<FS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("root_directory", &self.root_directory)
            .finish()
    }
}

impl<FS: FileSystem> PartialEq for Card<FS> {
    fn eq(&self, other: &Self) -> bool {
        self.root_directory == other.root_directory
    }
}

impl<FS: FileSystem> Card<FS> {
    /// Find the root directory of the card.
    ///
    /// Given a path, this function tries to find the root of a Deluge card
    /// by checking the presence of the required directories `KITS`, `SYNTHS` and `SAMPLES`.
    pub fn find_root_card_directory(file_system: &FS, initial_path: &Path) -> Result<Option<PathBuf>, CardError> {
        let mut current_path = initial_path;

        loop {
            if Self::check_required_directories(file_system, current_path).is_ok() {
                return Ok(Some(current_path.to_path_buf()));
            }

            match current_path.parent() {
                Some(parent) => current_path = parent,
                None => return Ok(None),
            }
        }
    }

    /// Check the required directories exist, return an error if not.
    fn check_required_directories(file_system: &FS, root_directory: &Path) -> Result<(), CardError> {
        let directory_names = file_system
            .get_directory_entries(root_directory)?
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .map(|file_name| file_name.to_string_lossy().to_string())
            })
            .collect::<BTreeSet<String>>();

        for required_directory in CardFolder::iter() {
            if !directory_names.contains(required_directory.directory_name()) {
                return Err(CardError::MissingRootDirectory(
                    required_directory
                        .directory_name()
                        .to_owned(),
                ));
            }
        }

        Ok(())
    }

    /// Creates the card directory and the required folders.
    ///
    /// The root directory must exists otherwise an error is returned.
    /// The other directories may or may not exist, they will be created as needed.
    /// Existing files or folder excepted the standard ones are simply ignored.
    pub fn create(file_system: FS, root_directory: &Path) -> Result<Self, CardError> {
        let root_directory = root_directory.to_path_buf();

        if !file_system.directory_exists(&root_directory) {
            return Err(CardError::DirectoryDoesNotExists(root_directory));
        }

        let card = Self {
            file_system: Arc::new(file_system),
            root_directory,
        };

        for required_directory in CardFolder::iter() {
            let path = &card.get_directory_path(required_directory);

            if !card.file_system.directory_exists(path) {
                card.file_system
                    .create_directory(path)?;
            }
        }

        Ok(card)
    }

    /// Open a card directory.
    ///
    /// The folder structure is checked and an error is returned if something wrong is found.
    pub fn open(file_system: FS, root_directory: &Path) -> Result<Self, CardError> {
        let root_directory = root_directory.to_path_buf();

        if !file_system.directory_exists(&root_directory) {
            return Err(CardError::DirectoryDoesNotExists(root_directory));
        }

        Self::check_required_directories(&file_system, &root_directory)?;

        Ok(Self {
            file_system: Arc::new(file_system),
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
                path.strip_prefix(self.root_directory())
                    .unwrap_or_else(|e| panic!("strip prefix of '{:?}': {:?}", self.root_directory(), e))
                    .to_string_lossy(),
            )?),
            false => Err(CardError::FileNotInCard(path.to_path_buf())),
        }
    }

    /// Get the absolute path of a sample on the card
    pub fn absolute_path(&self, path: &SamplePath) -> PathBuf {
        self.root_directory
            .as_path()
            .join(path.to_path())
    }

    /// Get one of the card's directory path
    pub fn get_directory_path(&self, folder: CardFolder) -> PathBuf {
        self.root_directory
            .join(folder.directory_name())
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

        for path in &self
            .file_system
            .get_directory_entries(&self.get_directory_path(folder))?
        {
            if self.file_system.is_file(path)? {
                if let Some(file_name) = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                {
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
            number: max_number
                .map(|n| n + 1)
                .unwrap_or(0u16),
            suffix: None,
        }
        .to_string())
    }
}
