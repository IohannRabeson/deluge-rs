//! A crate to read and write Synthstrom Deluge's patches
//!
//! This crate provides the data structures [Synth] and [Kit] that represent a Deluge synth patch and a kit patch.
//! It hides the details from the user, like the fact there are at least differents version of the XML schema
//! for each patch types.
//!
//! This crate makes heavy use of the Rust type system to reduce the possibilities of error. There is almost
//! one type for each different fields. Each value type specifies how to serialize/deserialize and what is the default
//! value. Some types such as [Transpose] or [ClippingAmount] for example are strong integer with constrained range
//! to avoid overflows.
//!
//! Each structures of this crate can be created using the builder pattern.
//!
//!
//! https://docs.google.com/document/d/11DUuuE1LBYOVlluPA9McT1_dT4AofZ5jnUD5eHvj7Vs/edit

mod card;
mod kit;
mod serialization;
mod sound;
mod synth;
mod values;

pub use card::{Card, CardError, CardFolder, FileSystem, LocalFileSystem, PatchName};
pub use kit::{CvGateRow, Hpf, HpfBuilder, Kit, KitBuilder, Lpf, LpfBuilder, MidiRow, RowKit, SoundRow};
pub use serialization::{load_kit, load_synth, save_kit, save_synth, PatchType, SerializationError};
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
