//! A crate to create, read and write Synthstrom Deluge's patches
//!
//! This crate provides the data structures [Synth] and [Kit] that represent a Deluge synth patch and a kit patch.
//! It hides the details from the user like the differents version of the XML schema.
//!
//! #### Reading patches
//! The crate provide function to read a synth or a kit from a file:
//! ```no_run
//! let kit = deluge::read_kit_from_file("Your Card/KITS/YOUR_KIT.XML")?;
//! let synth = deluge::read_synth_from_file("Your Card/SYNTHS/YOUR_SYNTH.XML")?;
//! # Ok::<(), deluge::ReadError>(())
//! ```
//!
//! #### Writing patches
//! It's also possible to write patches. The following example demonstrate how
//! to create a default kit like the Deluge would do then save it to a file:
//! ```no_run
//! deluge::write_kit_to_file(&deluge::Kit::default(), "Your Card/KITS/KIT001.XML")?;
//! deluge::write_synth_to_file(&deluge::Synth::default(), "Your Card/SYNTHS/YOUR_SYNTH.XML")?;
//! # Ok::<(), deluge::WriteError>(())
//! ```
//!
//! #### Deluge's card structure
//! To help with the Deluge's card folders structure [Card] is provided. It allows to create a new card, check existing card structure
//! and get the paths of the important directories such as KITS and SAMPLES.
//! ```
//! # use std::path::Path;
//! # use deluge::{LocalFileSystem, PatchType, CardError, CardFolder};
//! if let Ok(card) = deluge::Card::open(LocalFileSystem::default(), Path::new("your card directory")) {
//!     println!("Kits directory: {:?}", card.get_directory_path(CardFolder::Kits));
//!     println!("Synths directory: {:?}", card.get_directory_path(CardFolder::Synths));
//! }
//! # Ok::<(), CardError>(())
//! ```
//!
//! #### Strong typing
//! This crate makes heavy use of the Rust type system to reduce the possibilities of error. There is almost
//! one type for each different fields. Each value type specifies how to serialize/deserialize and what is the default
//! value. Some types such as [Transpose] or [ClippingAmount] for example are strong integer with constrained range
//! to avoid overflows.
//!
//! Each structures of this crate can be created using the builder pattern.

mod card;
mod kit;
mod serialization;
mod sound;
mod synth;
mod values;

pub use card::{Card, CardError, CardFolder, FileSystem, LocalFileSystem, PatchName};
pub use kit::{CvGateRow, Hpf, HpfBuilder, Kit, KitBuilder, Lpf, LpfBuilder, MidiRow, RowKit, SoundRow};
pub use serialization::{
    deserialize_kit, deserialize_kit_with_version, deserialize_synth, deserialize_synth_with_version, serialize_kit,
    serialize_synth, PatchType, SerializationError, VersionInfo,
};
pub use sound::{
    Arpeggiator, ArpeggiatorBuilder, Chorus, ChorusBuilder, Delay, DelayBuilder, Distorsion, DistorsionBuilder, Envelope,
    EnvelopeBuilder, Equalizer, EqualizerBuilder, Flanger, FlangerBuilder, FmCarrier, FmCarrierBuilder, FmModulator,
    FmModulatorBuilder, FmSynth, FmSynthBuilder, Lfo1, Lfo1Builder, Lfo2, Lfo2Builder, ModKnob, ModKnobBuilder, ModulationFx,
    PatchCable, PatchCableBuilder, Phaser, PhaserBuilder, RingModSynth, Sample, SampleOneZone, SampleOscillator,
    SampleOscillatorBuilder, SampleRange, SampleZone, Sidechain, Sound, SoundBuilder, SubtractiveOscillator, SubtractiveSynth,
    SubtractiveSynthBuilder, SynthEngine, Unison, UnisonBuilder, WaveformOscillator, WaveformOscillatorBuilder,
};
pub use synth::Synth;
pub use values::{
    ArpeggiatorMode, AttackSidechain, ClippingAmount, CvGateChannel, DecU50, FilterType, FineTranspose, HexU50, LfoShape,
    LpfMode, MidiChannel, ModulationFxType, OctavesCount, OnOff, OscType, Pan, PitchSpeed, Polyphony, ReleaseSidechain,
    RetrigPhase, SamplePath, SamplePlayMode, SamplePosition, SyncLevel, SynthMode, TableIndex, TimeStretchAmount, Transpose,
    UnisonDetune, UnisonVoiceCount, VoicePriority,
};

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("Deserialization error: {0}")]
    DeserializationError(SerializationError),

    #[error("Error while reading: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Error while reading '{1}': {0}")]
    ReadFileError(std::io::Error, PathBuf),
}

impl ReadError {
    pub fn new_file_error<P: AsRef<Path>>(error: ReadError, path: P) -> ReadError {
        match error {
            ReadError::DeserializationError(e) => ReadError::DeserializationError(e),
            ReadError::ReadError(e) => ReadError::ReadFileError(e, path.as_ref().to_path_buf()),
            ReadError::ReadFileError(e, path) => ReadError::ReadFileError(e, path),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WriteError {
    #[error("Serialization error: {0}")]
    SerializationError(SerializationError),

    #[error("Error while writing: {0}")]
    WriteError(std::io::Error),

    #[error("Error while writing '{1}': {0}")]
    WriteFileError(std::io::Error, PathBuf),
}

impl WriteError {
    pub fn new_file_error<P: AsRef<Path>>(error: WriteError, path: P) -> WriteError {
        match error {
            WriteError::SerializationError(e) => WriteError::SerializationError(e),
            WriteError::WriteError(e) => WriteError::WriteFileError(e, path.as_ref().to_path_buf()),
            WriteError::WriteFileError(e, path) => WriteError::WriteFileError(e, path),
        }
    }
}

pub fn read_synth<R: Read>(read: &mut R) -> Result<Synth, ReadError> {
    let mut xml_content = String::new();

    read.read_to_string(&mut xml_content)
        .map_err(ReadError::ReadError)?;

    deserialize_synth(&xml_content).map_err(ReadError::DeserializationError)
}

pub fn read_synth_with_version<R: Read>(read: &mut R) -> Result<(Synth, VersionInfo), ReadError> {
    let mut xml_content = String::new();

    read.read_to_string(&mut xml_content)
        .map_err(ReadError::ReadError)?;

    deserialize_synth_with_version(&xml_content).map_err(ReadError::DeserializationError)
}

pub fn read_synth_from_file<P: AsRef<Path>>(path: P) -> Result<Synth, ReadError> {
    let mut file = std::fs::File::open(&path).map_err(|e| ReadError::ReadFileError(e, path.as_ref().to_path_buf()))?;

    read_synth(&mut file).map_err(|e| ReadError::new_file_error(e, path.as_ref()))
}

pub fn read_synth_from_file_with_version<P: AsRef<Path>>(path: P) -> Result<(Synth, VersionInfo), ReadError> {
    let mut file = std::fs::File::open(&path).map_err(|e| ReadError::ReadFileError(e, path.as_ref().to_path_buf()))?;

    read_synth_with_version(&mut file).map_err(|e| ReadError::new_file_error(e, path.as_ref()))
}

pub fn read_kit<R: Read>(read: &mut R) -> Result<Kit, ReadError> {
    let mut xml_content = String::new();

    read.read_to_string(&mut xml_content)
        .map_err(ReadError::ReadError)?;

    deserialize_kit(&xml_content).map_err(ReadError::DeserializationError)
}

pub fn read_kit_with_version<R: Read>(read: &mut R) -> Result<(Kit, VersionInfo), ReadError> {
    let mut xml_content = String::new();

    read.read_to_string(&mut xml_content)
        .map_err(ReadError::ReadError)?;

    deserialize_kit_with_version(&xml_content).map_err(ReadError::DeserializationError)
}

pub fn read_kit_from_file<P: AsRef<Path>>(path: P) -> Result<Kit, ReadError> {
    let mut file = std::fs::File::open(&path).map_err(|e| ReadError::ReadFileError(e, path.as_ref().to_path_buf()))?;

    read_kit(&mut file).map_err(|e| ReadError::new_file_error(e, path.as_ref()))
}

pub fn read_kit_from_file_with_version<P: AsRef<Path>>(path: P) -> Result<(Kit, VersionInfo), ReadError> {
    let mut file = std::fs::File::open(&path).map_err(|e| ReadError::ReadFileError(e, path.as_ref().to_path_buf()))?;

    read_kit_with_version(&mut file).map_err(|e| ReadError::new_file_error(e, path.as_ref()))
}

pub fn write_synth<W: Write>(synth: &Synth, writable: &mut W) -> Result<(), WriteError> {
    let xml_content = serialize_synth(synth).map_err(WriteError::SerializationError)?;

    writable
        .write_all(xml_content.as_bytes())
        .map_err(WriteError::WriteError)
}

pub fn write_synth_to_file<P: AsRef<Path>>(synth: &Synth, path: P) -> Result<(), WriteError> {
    let mut file = std::fs::File::create(&path).map_err(|e| WriteError::WriteFileError(e, path.as_ref().to_path_buf()))?;

    write_synth(synth, &mut file).map_err(|e| WriteError::new_file_error(e, path))
}

pub fn write_kit<W: Write>(kit: &Kit, writable: &mut W) -> Result<(), WriteError> {
    let xml_content = serialize_kit(kit).map_err(WriteError::SerializationError)?;

    writable
        .write_all(xml_content.as_bytes())
        .map_err(WriteError::WriteError)
}

pub fn write_kit_to_file<P: AsRef<Path>>(kit: &Kit, path: P) -> Result<(), WriteError> {
    let mut file = std::fs::File::create(&path).map_err(|e| WriteError::WriteFileError(e, path.as_ref().to_path_buf()))?;

    write_kit(kit, &mut file).map_err(|e| WriteError::new_file_error(e, path))
}
