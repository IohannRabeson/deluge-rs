//! A crate to exchange with Synthstrom Deluge
//!
//! This crate provides the data structures Sound and Kit. You can read and write them using the XML deluge schema.
//! It hides the crap from the user, like the fact there are at least 3 differents version of the XML schema.
//! https://docs.google.com/document/d/11DUuuE1LBYOVlluPA9McT1_dT4AofZ5jnUD5eHvj7Vs/edit

#[macro_use]
extern crate derivative;

mod kit;
mod serialization;
mod synth;
mod sound;
mod values;

use std::{num::ParseIntError, sync::Arc};

pub use kit::{GateOutput, Kit, MidiOutput, SoundSource};
pub use sound::{
    Arpeggiator, Chorus, Delay, Distorsion, Envelope, Equalizer, Flanger, FmCarrier, FmGenerator, FmModulator, Lfo1, Lfo2,
    ModKnob, ModulationFx, Oscillator, PatchCable, Phaser, RingModGenerator, Sample, SampleOneZone, SampleOscillator,
    SamplePosition, SampleRange, SampleZone, Sidechain, Sound, SoundGenerator, SubtractiveGenerator, Unison, WaveformOscillator,
};
pub use synth::Synth;

pub use serialization::{load_kit, load_synth, save_kit, save_synth};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("parsing XML failed: {0}")]
    XmlParsingFailed(#[from] Arc<xmltree::ParseError>),

    #[error("parsing integer failed: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("parsing error: {0}")]
    SerdeError(#[from] serde_plain::Error),

    #[error("missing attribute '{1}' expected in parent '{0}'")]
    MissingAttribute(String, String),

    #[error("missing element '{0}'")]
    MissingElement(String),

    #[error("missing child '{1}' expected in parent '{0}")]
    MissingChild(String, String),

    #[error("missing attribute and child '{1}' expected in parent '{0}'")]
    MissingChildOrAttribute(String, String),

    #[error("loading failed: {0}")]
    LoadingFailed(String),

    #[error("unsupported sound source '{0}'")]
    UnsupportedSoundSource(String),

    #[error("unsupported sound type")]
    UnsupportedSoundType,

    #[error("invalid version format")]
    InvalidVersionFormat,

    #[error("unsupported version")]
    UnsupportedVersion,

    #[error("overflow: {0} > {1}")]
    Overflow(String, String),

    #[error("underflow: {0} < {1}")]
    Underflow(String, String),

    #[error("invalid hexadecimal u32 '{0}': {1}")]
    ParseHexdecimalU32Error(String, std::num::ParseIntError),

    #[error("invalid i32 '{0}': {1}")]
    ParseI32Error(String, std::num::ParseIntError),

    #[error("conversion error: {0}")]
    ConversionError(#[from] Arc<std::io::Error>),

    #[error("unsupported modulation fx: {0}")]
    UnsupportedModulationFx(String),

    #[error("value not found in table: {0}")]
    ValueNotFoundInTable(u32),

    #[error("unsupported sample type")]
    UnsupportedSampleType,
}

#[cfg(test)]
mod tests {
    fn check_sync<T: Sync>() {
        // Does nothing
    }

    #[test]
    fn test_error_is_sync() {
        check_sync::<super::Error>();
    }
}
