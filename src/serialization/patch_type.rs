use std::str::FromStr;

use crate::CardFolder;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PatchType {
    Synth,
    Kit,
}

impl PatchType {
    pub fn get_key<'a>(self) -> &'a str {
        match self {
            PatchType::Kit => KIT_KEY,
            PatchType::Synth => SYNTH_KEY,
        }
    }

    pub fn get_standard_patch_base_name<'a>(self) -> &'a str {
        match self {
            PatchType::Kit => KIT_BASE_NAME,
            PatchType::Synth => SYNTH_BASE_NAME,
        }
    }

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
