use std::collections::BTreeSet;

use crate::{
    values::{
        ArpeggiatorMode, DecU50, FineTranspose, HexU50, OctavesCount, OscType, Pan, Polyphony, RetrigPhase, SamplePath,
        SyncLevel, SynthMode, Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
    },
    SamplePosition,
};

use enum_as_inner::EnumAsInner;

mod effects;
mod fm;
mod modulators;
mod ring_mod;
mod subtractive;

pub use effects::{
    Chorus, ChorusBuilder, Delay, DelayBuilder, Distorsion, DistorsionBuilder, Equalizer, EqualizerBuilder, Flanger,
    FlangerBuilder, ModulationFx, Phaser, PhaserBuilder, Sidechain, SidechainBuilder,
};

pub use fm::{FmCarrier, FmCarrierBuilder, FmModulator, FmModulatorBuilder, FmSynth, FmSynthBuilder};
pub use modulators::{
    Envelope, EnvelopeBuilder, Lfo1, Lfo1Builder, Lfo2, Lfo2Builder, ModKnob, ModKnobBuilder, PatchCable, PatchCableBuilder,
};
pub use ring_mod::{RingModSynth, RingModSynthBuilder};
pub use subtractive::{
    Sample, SampleOneZone, SampleOneZoneBuilder, SampleOscillator, SampleOscillatorBuilder, SampleRange, SampleRangeBuilder,
    SampleZone, SampleZoneBuilder, SubtractiveOscillator, SubtractiveSynth, SubtractiveSynthBuilder,
};

/// Composes Synth and Kit patches
///
/// [Sound] is the main component of [Synth] patches. It's also the main component of [Kit], in this case
/// a [Sound] is wrapped in a [RowKit].
/// 
/// This crate provides [SoundBuilder] for creating [Sound] instances:
/// ```
/// # use deluge::{SoundBuilder, Sound, SubtractiveOscillator, SubtractiveSynthBuilder, Sample, SynthEngine, SamplePath};
/// # let path = SamplePath::new("path/to file.wav").unwrap();
/// # let generator = SubtractiveSynthBuilder::default()
/// #    .osc1(SubtractiveOscillator::new_sample(Sample::new(path, 0.into(), 1000.into())))
/// #    .osc2(SubtractiveOscillator::new_sample(Sample::default()))
/// #    .osc2_volume(0.into())
/// #    .build()
/// #    .unwrap();
/// let sound = SoundBuilder::default()
///     .generator(SynthEngine::from(generator))
///     .build()
///     .unwrap();
/// ```
/// [RowKit]: crate::RowKit
/// [Synth]: crate::Synth
/// [Kit]: crate::Kit
/// [SoundBuilder]: crate::SoundBuilder
#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct Sound {
    pub generator: SynthEngine,
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

    #[builder(setter(each(name = "add_cable")))]
    pub cables: Vec<PatchCable>,

    // This must be an array
    pub mod_knobs: Vec<ModKnob>,
}

impl Sound {
    /// Factory function that creates a regular sample based sound
    pub fn new_sample(path: SamplePath, start: SamplePosition, end: SamplePosition) -> Self {
        let generator = SubtractiveSynthBuilder::default()
            .osc1(SubtractiveOscillator::new_sample(Sample::new(path, start, end)))
            .osc2(SubtractiveOscillator::new_sample(Sample::default()))
            .osc2_volume(0.into())
            .build()
            .unwrap();

        Self {
            generator: SynthEngine::from(generator),
            ..Default::default()
        }
    }

    pub fn new_substractive(osc1: SubtractiveOscillator, osc2: SubtractiveOscillator) -> Self {
        Self {
            generator: SynthEngine::from(SubtractiveSynth::new(osc1, osc2)),
            ..Default::default()
        }
    }

    pub fn new_ringmod(osc1: WaveformOscillator, osc2: WaveformOscillator) -> Self {
        Self {
            generator: SynthEngine::from(RingModSynth::new(osc1, osc2)),
            ..Default::default()
        }
    }

    pub fn new_fm(carrier1: FmCarrier, carrier2: FmCarrier) -> Self {
        Self {
            generator: SynthEngine::from(FmSynth::new(carrier1, carrier2)),
            ..Default::default()
        }
    }

    /// Gets all the sample paths used by this sound.
    pub fn get_sample_paths(&self) -> BTreeSet<SamplePath> {
        let mut paths = BTreeSet::new();

        if let SynthEngine::Subtractive(generator) = &self.generator {
            if let SubtractiveOscillator::Sample(generator) = &generator.osc1 {
                paths.extend(generator.sample.get_sample_paths().into_iter());
            }

            if let SubtractiveOscillator::Sample(generator) = &generator.osc2 {
                paths.extend(generator.sample.get_sample_paths().into_iter());
            }
        }

        paths
    }
}

/// Default implementation for Sound
///
/// This implementation returns a Sound exactly like the
/// Deluge would create it for a default synth patch.
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

/// The synth mode
///
/// Each value contains a struct specific to each mode.
///
/// Using [From] is the easiest way to instanciate a [SynthEngine]
/// ```
/// use deluge::{SynthEngine, SubtractiveSynth, RingModSynth, FmSynth};
/// let subtractive_synth_mode = SynthEngine::from(SubtractiveSynth::default());
/// let ring_mod_synth_mode = SynthEngine::from(RingModSynth::default());
/// let fm_synth_mode = SynthEngine::from(FmSynth::default());
/// ```
#[derive(Clone, Debug, PartialEq, EnumAsInner)]
pub enum SynthEngine {
    Subtractive(SubtractiveSynth),
    RingMod(RingModSynth),
    Fm(FmSynth),
}

impl From<SubtractiveSynth> for SynthEngine {
    fn from(synth: SubtractiveSynth) -> Self {
        SynthEngine::Subtractive(synth)
    }
}

impl From<RingModSynth> for SynthEngine {
    fn from(synth: RingModSynth) -> Self {
        SynthEngine::RingMod(synth)
    }
}

impl From<FmSynth> for SynthEngine {
    fn from(synth: FmSynth) -> Self {
        SynthEngine::Fm(synth)
    }
}

impl SynthEngine {
    pub fn to_sound_type(&self) -> SynthMode {
        match self {
            SynthEngine::Subtractive(_) => SynthMode::Subtractive,
            SynthEngine::Fm(_) => SynthMode::Fm,
            SynthEngine::RingMod(_) => SynthMode::RingMod,
        }
    }
}

/// Implementation by default is the default [SubtractiveSynth]
impl Default for SynthEngine {
    fn default() -> Self {
        SynthEngine::Subtractive(SubtractiveSynth::default())
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct WaveformOscillator {
    pub osc_type: OscType,
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub pulse_width: HexU50,
}

impl WaveformOscillator {
    pub fn new_sine() -> Self {
        Self {
            osc_type: OscType::Sine,
            ..Default::default()
        }
    }
    pub fn new_triangle() -> Self {
        Self {
            osc_type: OscType::Triangle,
            ..Default::default()
        }
    }
    pub fn new_saw() -> Self {
        Self {
            osc_type: OscType::Saw,
            ..Default::default()
        }
    }
    pub fn new_square() -> Self {
        Self {
            osc_type: OscType::Square,
            ..Default::default()
        }
    }
}

impl Default for WaveformOscillator {
    fn default() -> Self {
        Self {
            osc_type: OscType::Sine,
            transpose: Default::default(),
            fine_transpose: Default::default(),
            retrig_phase: Default::default(),
            pulse_width: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
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

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
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
