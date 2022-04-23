use crate::values::{HexU50, LfoShape, SyncLevel};

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct Envelope {
    pub attack: HexU50,
    pub decay: HexU50,
    pub sustain: HexU50,
    pub release: HexU50,
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct Lfo1 {
    pub shape: LfoShape,
    pub sync_level: SyncLevel,
    pub rate: HexU50,
}

impl Default for Lfo1 {
    fn default() -> Self {
        Self {
            shape: LfoShape::Triangle,
            sync_level: SyncLevel::Off,
            rate: 30.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct Lfo2 {
    pub shape: LfoShape,
    pub rate: HexU50,
}

impl Default for Lfo2 {
    fn default() -> Self {
        Self {
            shape: LfoShape::Triangle,
            rate: 25.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct PatchCable {
    pub source: String,
    pub destination: String,
    pub amount: HexU50,
}

impl PatchCable {
    pub fn new(source: &str, destination: &str, amount: HexU50) -> Self {
        Self {
            source: source.to_string(),
            destination: destination.to_string(),
            amount,
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct ModKnob {
    pub control_param: String,
    pub patch_amount_from_source: Option<String>,
}

impl ModKnob {
    pub fn new(control_param: &str) -> Self {
        Self {
            control_param: control_param.to_string(),
            patch_amount_from_source: None,
        }
    }

    pub fn new_with_patch_amount(control_param: &str, patch_amount_from_source: &str) -> Self {
        Self {
            control_param: control_param.to_string(),
            patch_amount_from_source: Some(patch_amount_from_source.to_string()),
        }
    }
}
