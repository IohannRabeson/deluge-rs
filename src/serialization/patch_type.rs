use std::str::FromStr;

use crate::CardFolder;

/// The type of a patch.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PatchType {
    /// Synth patch type.
    Synth,
    /// Kit patch type.
    Kit,
}

impl PatchType {
    /// Get the XML key specifying a patch type.
    pub fn get_key<'a>(self) -> &'a str {
        match self {
            PatchType::Kit => KIT_KEY,
            PatchType::Synth => SYNTH_KEY,
        }
    }

    /// Get the standard base name.
    pub fn get_standard_base_name<'a>(self) -> &'a str {
        match self {
            PatchType::Kit => KIT_BASE_NAME,
            PatchType::Synth => SYNTH_BASE_NAME,
        }
    }

    /// Get the patch folder.
    pub fn get_card_folder(self) -> CardFolder {
        match self {
            PatchType::Kit => CardFolder::Kits,
            PatchType::Synth => CardFolder::Synths,
        }
    }
}

const KIT_KEY: &str = "kit";
const SYNTH_KEY: &str = "sound";
const KIT_BASE_NAME: &str = "KIT";
const SYNTH_BASE_NAME: &str = "SYNT";

impl FromStr for PatchType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            KIT_BASE_NAME => Ok(PatchType::Kit),
            SYNTH_BASE_NAME => Ok(PatchType::Synth),
            _ => Err(()),
        }
    }
}
