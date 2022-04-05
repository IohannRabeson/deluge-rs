use std::path::{Path, PathBuf};

use super::CardError;

use mockall::predicate::*;
use mockall::*;

/// This trait exists to make unit testing possible.

#[automock]
pub trait FileSystem {
    /// This method gives the paths of the directories present in a given directory.
    fn get_directories(&self, path: &Path) -> Result<Vec<PathBuf>, CardError>;

    /// This method creates all the missing directories.
    fn create_directory(&self, path: &Path) -> Result<(), CardError>;

    /// Check if a directory exists
    fn directory_exists(&self, path: &Path) -> bool;
}

#[derive(Default)]
pub struct LocalFileSystem;

fn make_io_error(error: std::io::Error) -> CardError {
    CardError::IoError(error.to_string())
}

impl FileSystem for LocalFileSystem {
    fn get_directories(&self, path: &Path) -> Result<Vec<PathBuf>, CardError> {
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
}

// #[cfg(test)]
// pub mod tests
// {
//     use std::{path::{PathBuf, Path}, collections::BTreeMap};

//     use crate::card::CardError;

//     use super::FileSystem;

//     #[derive(Default)]
//     pub struct MockFileSystem<'a> {
//         directories_children: BTreeMap<&'a Path, Vec<&'a Path>>,
//         existing_
//     }

//     impl<'a> MockFileSystem<'a> {
//         pub fn add_children_directory(&mut self, parent: &'a Path, child: &'a Path) {
//             self.directories_children.entry(parent).or_insert(Vec::new()).push(child);
//         }
//     }

//     impl<'a> FileSystem for MockFileSystem<'a> {
//         fn get_directories(&self, path: &Path) -> Result<Vec<PathBuf>, CardError> {
//             Ok(match self.directories_children.get(path) {
//                 Some(children) => children.into_iter().map(|p|p.to_path_buf()).collect::<Vec<PathBuf>>(),
//                 None => Vec::new(),
//             })
//         }

//         fn create_directory(&self, _path: &Path) -> Result<(), CardError> {
//             Ok(())
//         }

//         fn directory_exists(&self, path: &Path) -> bool {
//             path.exists() && path.is_dir()
//         }
//     }
// }
