use std::path::{Path, PathBuf};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::CardError;

/// Path relative to a card.
#[derive(Clone, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub struct SamplePath(PathBuf);

impl SamplePath {
    /// Create a new sample path.
    ///
    /// This function returns an error if the path is not a relative one.
    pub fn new(path: &str) -> Result<Self, CardError> {
        let path = Path::new(path);

        if !path.is_relative() {
            return Err(CardError::PathNotRelative(path.to_path_buf()));
        }

        Ok(SamplePath(path.to_path_buf()))
    }

    pub fn to_string_lossy(&self) -> String {
        use itertools::Itertools;

        self.0.components().map(|c| c.as_os_str().to_string_lossy()).join("/")
    }

    pub(crate) fn to_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl Serialize for SamplePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string_lossy())
    }
}

struct PathVisitor;

impl<'de> Visitor<'de> for PathVisitor {
    type Value = SamplePath;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("sample path relative to the card root directory")
    }

    fn visit_str<E>(self, text: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        SamplePath::new(text).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for SamplePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PathVisitor)
    }
}
