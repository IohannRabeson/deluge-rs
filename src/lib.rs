//! A crate to read and write Synthstrom Deluge's patches
//!
//! This crate provides the data structures Sound and Kit. You can read and write them using the XML deluge schema.
//! It hides the crap from the user, like the fact there are at least differents version of the XML schema.
//!
//! # Data structures
//! There are 2 types of patches: synth and kit.  
//! A synth contains one sound, a kit contains [1-n] sounds.  
//! More precisely, a kit contains named sounds. Other type of row
//! are not named.
//!
//! https://docs.google.com/document/d/11DUuuE1LBYOVlluPA9McT1_dT4AofZ5jnUD5eHvj7Vs/edit

mod card;
mod kit;
mod serialization;
mod sound;
mod synth;
mod values;

pub use card::{Card, CardError, CardFolder, FileSystem, LocalFileSystem, PatchName};
pub use kit::{AudioOutput, CvGateOutput, Hpf, Kit, Lpf, MidiOutput, RowKit};
pub use sound::{
    Arpeggiator, Chorus, Delay, Distorsion, Envelope, Equalizer, Flanger, FmCarrier, FmGenerator, FmModulator, Lfo1, Lfo2,
    ModKnob, ModulationFx, Oscillator, PatchCable, Phaser, RingModGenerator, Sample, SampleOneZone, SampleOscillator,
    SamplePosition, SampleRange, SampleZone, Sidechain, Sound, SoundGenerator, SubtractiveGenerator, Unison, WaveformOscillator,
};
pub use synth::Synth;

pub use serialization::{load_kit, load_synth, save_kit, save_synth, PatchType, SerializationError};
