use crate::values::{
    ArpeggiatorMode, AttackSidechain, ClippingAmount, DecU50, FineTranspose, HexU50, LfoShape, LpfMode, OctavesCount, OnOff,
    OscType, Pan, PitchSpeed, Polyphony, ReleaseSidechain, RetrigPhase, SamplePlayMode, SoundType, SyncLevel, TableIndex,
    TimeStretchAmount, Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
};
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
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

impl Sound {
    pub fn new_substractive(osc1: Oscillator, osc2: Oscillator) -> Self {
        Self {
            generator: SoundGenerator::Subtractive(SubtractiveGenerator::new(osc1, osc2)),
            ..Default::default()
        }
    }

    pub fn new_sample(sample: Sample) -> Self {
        let mut osc2 = Oscillator::new_sample(Sample::OneZone(SampleOneZone {
            file_path: String::new(),
            zone: None,
        }));

        osc2.set_volume(0.into());
        
        Self {
            generator: SoundGenerator::Subtractive(SubtractiveGenerator::new(Oscillator::new_sample(sample), osc2)),
            ..Default::default()
        }
    }

    pub fn new_ringmod(osc1: Oscillator, osc2: Oscillator) -> Self {
        Self {
            generator: SoundGenerator::RingMod(RingModGenerator::new(osc1, osc2)),
            ..Default::default()
        }
    }

    pub fn new_fm(carrier1: FmCarrier, carrier2: FmCarrier) -> Self {
        Self {
            generator: SoundGenerator::Fm(FmGenerator::new(carrier1, carrier2)),
            ..Default::default()
        }
    }
}

/// Default implementation for Sound
///
/// This implementation returns a Sound exactly like the Deluge would create it for a default synth patch.
impl Default for Sound {
    fn default() -> Self {
        let envelope1 = Envelope {
            attack: 0.into(),
            decay: 20.into(),
            sustain: 50.into(),
            release: 0.into(),
        };

        let envelope2 = Envelope {
            attack: 20.into(),
            decay: 20.into(),
            sustain: 25.into(),
            release: 20.into(),
        };

        let mod_knobs = vec![
            ModKnob::new("pan"),
            ModKnob::new("volumePostFX"),
            ModKnob::new("lpfResonance"),
            ModKnob::new("lpfFrequency"),
            ModKnob::new("env1Release"),
            ModKnob::new("env1Attack"),
            ModKnob::new("delayFeedback"),
            ModKnob::new("delayRate"),
            ModKnob::new("reverbAmount"),
            ModKnob::new_with_patch_amount("volumePostReverbSend", "compressor"),
            ModKnob::new_with_patch_amount("pitch", "lfo1"),
            ModKnob::new("lfo1Rate"),
            ModKnob::new("portamento"),
            ModKnob::new("stutterRate"),
            ModKnob::new("bitcrushAmount"),
            ModKnob::new("sampleRateReduction"),
        ];

        let cables = vec![PatchCable::new("velocity", "volume", 37.into())];

        Self {
            generator: Default::default(),
            polyphonic: Polyphony::Poly,
            voice_priority: Default::default(),
            volume: 40.into(),
            pan: Default::default(),
            portamento: 0.into(),
            reverb_amount: 0.into(),
            stutter_rate: 25.into(),
            sidechain_send: None,
            envelope1,
            envelope2,
            lfo1: Default::default(),
            lfo2: Default::default(),
            unison: Default::default(),
            arpeggiator: Arpeggiator::default(),
            delay: Delay::default(),
            distorsion: Distorsion::default(),
            modulation_fx: ModulationFx::Off,
            equalizer: Equalizer::default(),
            sidechain: Sidechain::default(),
            cables,
            mod_knobs,
        }
    }
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

impl Default for Lfo1 {
    fn default() -> Self {
        Self {
            shape: LfoShape::Triangle,
            sync_level: SyncLevel::Off,
            rate: 30.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq, EnumAsInner)]
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

impl Default for SoundGenerator {
    fn default() -> Self {
        SoundGenerator::Subtractive(SubtractiveGenerator::default())
    }
}

#[derive(Clone, Debug, PartialEq, EnumAsInner)]
pub enum Oscillator {
    Waveform(WaveformOscillator),
    Sample(SampleOscillator),
}

impl Oscillator {
    pub fn new_waveform(waveform: WaveformOscillator) -> Self {
        Oscillator::Waveform(waveform)
    }

    pub fn new_sample(sample: Sample) -> Self {
        Oscillator::Sample(SampleOscillator::new(sample))
    }

    pub fn set_volume(&mut self, volume: HexU50) {
        match self {
            Oscillator::Waveform(osc) => osc.volume = volume,
            Oscillator::Sample(osc) => osc.volume = volume,
        }
    }
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

impl SampleOscillator {
    pub fn new(sample: Sample) -> Self {
        Self {
            sample,
            ..Default::default()
        }
    }
}
impl Default for SampleOscillator {
    fn default() -> Self {
        Self {
            transpose: Default::default(),
            fine_transpose: Default::default(),
            mode: SamplePlayMode::Cut,
            reversed: OnOff::Off,
            pitch_speed: PitchSpeed::Independent,
            time_stretch_amount: Default::default(),
            linear_interpolation: OnOff::Off,
            volume: 50.into(),
            sample: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum Sample {
    OneZone(SampleOneZone),
    SampleRanges(Vec<SampleRange>),
}

impl Sample {
    pub fn new(file_path: &str, start: SamplePosition, end: SamplePosition) -> Self {
        Self::OneZone(SampleOneZone {
            file_path: file_path.to_string(),
            zone: Some(SampleZone {
                start,
                end,
                start_loop: None,
                end_loop: None,
            }),
        })
    }
}

impl Default for Sample {
    fn default() -> Self {
        Sample::new("", SamplePosition::new(0), SamplePosition::new(9999999))
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
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

impl SubtractiveGenerator {
    pub fn new(osc1: Oscillator, osc2: Oscillator) -> Self {
        Self {
            osc1,
            osc2,
            ..Default::default()
        }
    }
}

impl Default for SubtractiveGenerator {
    fn default() -> Self {
        let osc1 = Oscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            volume: 50.into(),
            pulse_width: 25.into(),
        });

        let osc2 = Oscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            volume: 0.into(),
            pulse_width: 25.into(),
        });

        Self {
            osc1,
            osc2,
            osc2_sync: OnOff::Off,
            noise: 0.into(),
            lpf_mode: LpfMode::Lpf24,
            lpf_frequency: 50.into(),
            lpf_resonance: 0.into(),
            hpf_frequency: 0.into(),
            hpf_resonance: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RingModGenerator {
    pub osc1: Oscillator,
    pub osc2: Oscillator,
    pub osc2_sync: OnOff,
    pub noise: HexU50,
}

impl RingModGenerator {
    pub fn new(osc1: Oscillator, osc2: Oscillator) -> Self {
        Self {
            osc1,
            osc2,
            ..Default::default()
        }
    }
}

impl Default for RingModGenerator {
    fn default() -> Self {
        let osc1 = Oscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            volume: 50.into(),
            pulse_width: 25.into(),
        });

        let osc2 = Oscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            volume: 0.into(),
            pulse_width: 25.into(),
        });

        Self {
            osc1,
            osc2,
            osc2_sync: OnOff::Off,
            noise: 0.into(),
        }
    }
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

impl FmGenerator {
    pub fn new(osc1: FmCarrier, osc2: FmCarrier) -> Self {
        Self {
            osc1,
            osc2,
            modulator1: FmModulator::default(),
            modulator2: FmModulator::default(),
            modulator2_to_modulator1: OnOff::Off,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FmCarrier {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub volume: HexU50,
    pub feedback: HexU50,
}

impl Default for FmCarrier {
    fn default() -> Self {
        Self {
            transpose: Default::default(),
            fine_transpose: Default::default(),
            retrig_phase: Default::default(),
            volume: 50.into(),
            feedback: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FmModulator {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub amount: HexU50,
    pub feedback: HexU50,
}

impl Default for FmModulator {
    fn default() -> Self {
        Self {
            transpose: Default::default(),
            fine_transpose: Default::default(),
            retrig_phase: RetrigPhase::Off,
            amount: 0.into(),
            feedback: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unison {
    pub voice_count: UnisonVoiceCount,
    pub detune: UnisonDetune,
}

impl Default for Unison {
    fn default() -> Self {
        Self {
            voice_count: 1.into(),
            detune: 8.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Delay {
    pub ping_pong: OnOff,
    pub analog: OnOff,
    pub amount: HexU50,
    pub rate: HexU50,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Arpeggiator {
    pub mode: ArpeggiatorMode,
    pub gate: HexU50,
    pub rate: HexU50,
    pub sync_level: SyncLevel,
    pub octaves_count: OctavesCount,
}

impl Default for Arpeggiator {
    fn default() -> Self {
        Self {
            mode: ArpeggiatorMode::Off,
            gate: 25.into(),
            rate: 25.into(),
            sync_level: SyncLevel::Sixteenth,
            octaves_count: 2.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl Default for Equalizer {
    fn default() -> Self {
        Self {
            bass_level: 25.into(),
            bass_frequency: 25.into(),
            treble_level: 25.into(),
            treble_frequency: 25.into(),
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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
