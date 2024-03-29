use enum_as_inner::EnumAsInner;

use crate::{
    values::{
        FineTranspose, HexU50, LpfMode, OnOff, OscType, PitchSpeed, RetrigPhase, SamplePath, SamplePlayMode, SamplePosition,
        TimeStretchAmount, Transpose,
    },
    WaveformOscillator,
};

/// Subtractive oscillator
///
/// To create an instance, you can use [From]:
/// ```
/// # use deluge::{
/// #    SubtractiveOscillator, WaveformOscillator, SampleOscillator,
/// #    WaveformOscillatorBuilder, SampleOscillatorBuilder, OscType
/// # };
/// let oscillator_1 = SubtractiveOscillator::from(WaveformOscillatorBuilder::default()
///     .osc_type(OscType::Sine)
///     .build().unwrap());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, EnumAsInner)]
pub enum SubtractiveOscillator {
    Waveform(WaveformOscillator),
    Sample(SampleOscillator),
}

impl SubtractiveOscillator {
    pub fn new_waveform(waveform: WaveformOscillator) -> Self {
        SubtractiveOscillator::Waveform(waveform)
    }

    pub fn new_sample(sample: Sample) -> Self {
        SubtractiveOscillator::Sample(SampleOscillator::new(sample))
    }
}

impl From<WaveformOscillator> for SubtractiveOscillator {
    fn from(oscillator: WaveformOscillator) -> Self {
        Self::Waveform(oscillator)
    }
}

impl From<SampleOscillator> for SubtractiveOscillator {
    fn from(oscillator: SampleOscillator) -> Self {
        Self::Sample(oscillator)
    }
}

/// Can be created using [SubtractiveSynthBuilder].
/// ```
/// use deluge::{SubtractiveSynthBuilder, WaveformOscillator};
///
/// let synth = SubtractiveSynthBuilder::default()
///     .osc1(WaveformOscillator::new_sine().into())
///     .osc2(WaveformOscillator::new_sine().into())
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
pub struct SubtractiveSynth {
    pub osc1: SubtractiveOscillator,
    pub osc2: SubtractiveOscillator,
    pub osc2_sync: OnOff,
    pub osc1_volume: HexU50,
    pub osc2_volume: HexU50,
    pub noise: HexU50,
    pub lpf_mode: LpfMode,
    pub lpf_frequency: HexU50,
    pub lpf_resonance: HexU50,
    pub hpf_frequency: HexU50,
    pub hpf_resonance: HexU50,
}

impl SubtractiveSynth {
    pub fn new(osc1: SubtractiveOscillator, osc2: SubtractiveOscillator) -> Self {
        Self {
            osc1,
            osc2,
            ..Default::default()
        }
    }
}

impl Default for SubtractiveSynth {
    fn default() -> Self {
        let osc1 = SubtractiveOscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            pulse_width: 25.into(),
        });

        let osc2 = SubtractiveOscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            pulse_width: 25.into(),
        });

        Self {
            osc1,
            osc2,
            osc2_sync: OnOff::Off,
            osc1_volume: 50.into(),
            osc2_volume: 0.into(),
            noise: 0.into(),
            lpf_mode: LpfMode::Lpf24,
            lpf_frequency: 50.into(),
            lpf_resonance: 0.into(),
            hpf_frequency: 0.into(),
            hpf_resonance: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
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
            sample: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum Sample {
    OneZone(SampleOneZone),
    SampleRanges(Vec<SampleRange>),
}

impl Sample {
    pub fn new(file_path: SamplePath, start: SamplePosition, end: SamplePosition) -> Self {
        Self::OneZone(SampleOneZone {
            file_path,
            zone: Some(SampleZone {
                start,
                end,
                start_loop: None,
                end_loop: None,
            }),
        })
    }

    pub fn get_sample_paths(&self) -> Vec<SamplePath> {
        match self {
            Sample::OneZone(zone) => Vec::from([zone.file_path.clone()]),
            Sample::SampleRanges(ranges) => Vec::from_iter(
                ranges
                    .iter()
                    .map(|range| range.file_path.clone()),
            ),
        }
    }
}

impl Default for Sample {
    fn default() -> Self {
        Sample::new(SamplePath::default(), 0u64.into(), 9999999u64.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, derive_builder::Builder)]
pub struct SampleOneZone {
    pub file_path: SamplePath,
    pub zone: Option<SampleZone>,
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
pub struct SampleRange {
    pub range_top_note: Option<u8>,
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub file_path: SamplePath,
    pub zone: SampleZone,
}

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
pub struct SampleZone {
    pub start: SamplePosition,
    pub end: SamplePosition,
    pub start_loop: Option<SamplePosition>,
    pub end_loop: Option<SamplePosition>,
}
