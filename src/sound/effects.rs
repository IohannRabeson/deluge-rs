use enum_as_inner::EnumAsInner;

use crate::values::{AttackSidechain, ClippingAmount, HexU50, OnOff, ReleaseSidechain, SyncLevel, TableIndex};

/// The delay parameters.
#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
pub struct Delay {
    /// Enable or disable the ping pong mode.
    pub ping_pong: OnOff,
    /// Enable or disable the analog mode.
    pub analog: OnOff,
    /// The amount of delay.
    pub amount: HexU50,
    /// The delay rate.
    pub rate: HexU50,
    /// The sync level.
    pub sync_level: SyncLevel,
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            ping_pong: OnOff::On,
            analog: OnOff::Off,
            amount: 0.into(),
            rate: 25.into(),
            sync_level: SyncLevel::Sixteenth,
        }
    }
}

/// The distorsion parameters.
#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
pub struct Distorsion {
    pub bit_crush: HexU50,
    pub saturation: ClippingAmount,
    pub decimation: HexU50,
}

impl Default for Distorsion {
    fn default() -> Self {
        Self {
            bit_crush: 0.into(),
            saturation: 0.into(),
            decimation: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
pub struct Equalizer {
    pub bass_level: HexU50,
    pub bass_frequency: HexU50,
    pub treble_level: HexU50,
    pub treble_frequency: HexU50,
}

impl Default for Equalizer {
    fn default() -> Self {
        Self {
            // The default must be HexU50(25)!
            // About 25 the basses are increased, below they are decreased
            bass_level: 25.into(),
            // Here again the default seems to be HexU50(25) but I'm not sure why
            bass_frequency: 25.into(),
            // The default must be HexU50(25)!
            // About 25 the treble are increased, below they are decreased
            treble_level: 25.into(),
            // Here again the default seems to be HexU50(25) but I'm not sure why
            treble_frequency: 25.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, EnumAsInner)]
pub enum ModulationFx {
    Off,
    Flanger(Flanger),
    Chorus(Chorus),
    Phaser(Phaser),
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
pub struct Flanger {
    pub rate: HexU50,
    pub feedback: HexU50,
}

impl Default for Flanger {
    fn default() -> Self {
        Self {
            rate: 25.into(),
            feedback: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
pub struct Chorus {
    pub rate: HexU50,
    pub depth: HexU50,
    pub offset: HexU50,
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
pub struct Phaser {
    pub rate: HexU50,
    pub depth: HexU50,
    pub feedback: HexU50,
}

/// Sidechain
///
/// Notice the "compressor" (the sidechain affecting the volume) is serialized
/// as a specific patch cable. When you edit the value accessible using the shortcut Row+Volduck this
/// is the amount of a patch cable.
#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
pub struct Sidechain {
    pub attack: AttackSidechain,
    pub release: ReleaseSidechain,
    pub shape: HexU50,
    pub sync: SyncLevel,
}

impl Default for Sidechain {
    fn default() -> Self {
        Self {
            attack: AttackSidechain::new(TableIndex::new(7)),
            release: ReleaseSidechain::new(TableIndex::new(28)),
            shape: 18.into(),
            sync: SyncLevel::Sixteenth,
        }
    }
}
