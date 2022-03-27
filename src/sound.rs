use crate::values::{
    ArpeggiatorMode, AttackSidechain, ClippingAmount, DecU50, FineTranspose, HexU50, LfoShape, LpfMode, OctavesCount, OnOff,
    OscType, Pan, PitchSpeed, Polyphony, ReleaseSidechain, RetrigPhase, SamplePlayMode, SoundType, SyncLevel, TimeStretchAmount,
    Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
};
use serde::{Deserialize, Serialize};

#[derive(Derivative, Clone, Debug)]
#[derivative(PartialEq)]
pub struct Sound {
    pub generator: SoundGenerator,
    pub polyphonic: Polyphony,
    pub voice_priority: VoicePriority,
    pub volume: HexU50,
    pub pan: Pan,
    pub portamento: HexU50,
    pub reverb_amount: HexU50,
    pub stutter_rate: HexU50,
    pub sidechain_send: Option<DecU50>,

    pub envelope1: Envelope,
    pub envelope2: Envelope,
    pub lfo1: Lfo1,
    pub lfo2: Lfo2,
    pub unison: Unison,
    pub arpeggiator: Arpeggiator,

    pub delay: Delay,
    pub distorsion: Distorsion,
    pub modulation_fx: ModulationFx,
    pub equalizer: Equalizer,
    pub sidechain: Sidechain,
    pub cables: Vec<PatchCable>,
    pub mod_knobs: Vec<ModKnob>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    pub attack: HexU50,
    pub decay: HexU50,
    pub sustain: HexU50,
    pub release: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lfo1 {
    pub shape: LfoShape,
    pub sync_level: SyncLevel,
    pub rate: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lfo2 {
    pub shape: LfoShape,
    pub rate: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum SoundGenerator {
    Subtractive(SubtractiveGenerator),
    RingMod(RingModGenerator),
    Fm(FmGenerator),
}

impl SoundGenerator {
    pub fn to_sound_type(&self) -> SoundType {
        match self {
            SoundGenerator::Subtractive(_) => SoundType::Subtractive,
            SoundGenerator::Fm(_) => SoundType::Fm,
            SoundGenerator::RingMod(_) => SoundType::RingMod,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum Oscillator {
    Waveform(WaveformOscillator),
    Sample(SampleOscillator),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaveformOscillator {
    pub osc_type: OscType,
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub volume: HexU50,
    pub pulse_width: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleOscillator {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub mode: SamplePlayMode,
    pub reversed: OnOff,
    pub pitch_speed: PitchSpeed,
    pub time_stretch_amount: TimeStretchAmount,
    /// When set to On, the low quality linear interpolation is used.
    /// The false Off enable high quality interpolation.
    pub linear_interpolation: OnOff,
    pub volume: HexU50,
    pub sample: Sample,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum Sample {
    OneZone(SampleOneZone),
    SampleRanges(Vec<SampleRange>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleOneZone {
    pub file_path: String,
    pub zone: Option<SampleZone>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleRange {
    pub range_top_note: Option<u8>,
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub file_path: String,
    pub zone: SampleZone,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SamplePosition(u64);

impl SamplePosition {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleZone {
    pub start: SamplePosition,
    pub end: SamplePosition,
    pub start_loop: Option<SamplePosition>,
    pub end_loop: Option<SamplePosition>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubtractiveGenerator {
    pub osc1: Oscillator,
    pub osc2: Oscillator,
    pub osc2_sync: OnOff,
    pub noise: HexU50,
    pub lpf_mode: LpfMode,
    pub lpf_frequency: HexU50,
    pub lpf_resonance: HexU50,
    pub hpf_frequency: HexU50,
    pub hpf_resonance: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RingModGenerator {
    pub osc1: Oscillator,
    pub osc2: Oscillator,
    pub osc2_sync: OnOff,
    pub noise: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FmGenerator {
    pub osc1: FmCarrier,
    pub osc2: FmCarrier,
    pub modulator1: FmModulator,
    pub modulator2: FmModulator,
    /// Parameter "Destination"
    /// If On modulator 2 modulates the modulator 1, otherwise it modulates the carrier 2.
    pub modulator2_to_modulator1: OnOff,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FmCarrier {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub volume: HexU50,
    pub feedback: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FmModulator {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub amount: HexU50,
    pub feedback: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unison {
    pub voice_count: UnisonVoiceCount,
    pub detune: UnisonDetune,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Delay {
    pub ping_pong: OnOff,
    pub analog: OnOff,
    pub amount: HexU50,
    pub rate: HexU50,
    pub sync_level: SyncLevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arpeggiator {
    pub mode: ArpeggiatorMode,
    pub gate: HexU50,
    pub rate: HexU50,
    pub sync_level: SyncLevel,
    pub octaves_count: OctavesCount,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Distorsion {
    pub bit_crush: HexU50,
    pub saturation: ClippingAmount,
    pub decimation: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Equalizer {
    /// The default must be HexU50(25)!
    /// About 25 the basses are increased, below they are decreased
    pub bass_level: HexU50,
    /// Here again the default seems to be HexU50(25) but I'm not sure why
    pub bass_frequency: HexU50,
    /// The default must be HexU50(25)!
    /// About 25 the treble are increased, below they are decreased
    pub treble_level: HexU50,
    /// Here again the default seems to be HexU50(25) but I'm not sure why
    pub treble_frequency: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModulationFx {
    Off,
    Flanger(Flanger),
    Chorus(Chorus),
    Phaser(Phaser),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Flanger {
    pub rate: HexU50,
    pub feedback: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chorus {
    pub rate: HexU50,
    pub depth: HexU50,
    pub offset: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct Sidechain {
    pub attack: AttackSidechain,
    pub release: ReleaseSidechain,
    pub shape: HexU50,
    pub sync: SyncLevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatchCable {
    pub source: String,
    pub destination: String,
    pub amount: HexU50,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModKnob {
    pub control_param: String,
    pub patch_amount_from_source: Option<String>,
}
