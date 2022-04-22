use std::collections::HashSet;

use crate::values::{
    ArpeggiatorMode, DecU50, FineTranspose, HexU50, OctavesCount, OscType, Pan, Polyphony, RetrigPhase, SamplePath, SoundType,
    SyncLevel, Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
};

use enum_as_inner::EnumAsInner;

mod effects;
mod fm;
mod modulators;
mod ring_mod;
mod subtractive;

pub use effects::{Chorus, Delay, Distorsion, Equalizer, Flanger, ModulationFx, Phaser, Sidechain};
pub use fm::{FmCarrier, FmGenerator, FmModulator};
pub use modulators::{Envelope, Lfo1, Lfo2, ModKnob, PatchCable};
pub use ring_mod::RingModGenerator;
pub use subtractive::{
    Sample, SampleOneZone, SampleOscillator, SampleRange, SampleZone, SubtractiveGenerator, SubtractiveOscillator,
};

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
    pub fn new_substractive(osc1: SubtractiveOscillator, osc2: SubtractiveOscillator) -> Self {
        Self {
            generator: SoundGenerator::Subtractive(SubtractiveGenerator::new(osc1, osc2)),
            ..Default::default()
        }
    }

    pub fn new_ringmod(osc1: WaveformOscillator, osc2: WaveformOscillator) -> Self {
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

    /// Gets all the sample paths in this sound.
    pub fn get_sample_paths(&self) -> HashSet<SamplePath> {
        let mut paths = HashSet::new();

        if let SoundGenerator::Subtractive(generator) = &self.generator {
            if let SubtractiveOscillator::Sample(generator) = &generator.osc1 {
                paths.extend(Self::get_sample_paths_impl(&generator.sample));
            }

            if let SubtractiveOscillator::Sample(generator) = &generator.osc2 {
                paths.extend(Self::get_sample_paths_impl(&generator.sample));
            }
        }

        paths
    }

    fn get_sample_paths_impl(sample: &Sample) -> Vec<SamplePath> {
        match sample {
            Sample::OneZone(zone) => Vec::from([zone.file_path.clone()]),
            Sample::SampleRanges(ranges) => Vec::from_iter(ranges.iter().map(|range| range.file_path.clone())),
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

#[derive(Clone, Debug, PartialEq)]
pub struct WaveformOscillator {
    pub osc_type: OscType,
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub pulse_width: HexU50,
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
