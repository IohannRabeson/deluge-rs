use crate::{
    values::{CvGateChannel, FilterType, LpfMode, MidiChannel, ModulationFxType, Polyphony, HexU50},
    Delay, Oscillator, Sample, SampleOneZone, SamplePosition, SampleZone, Sidechain, Equalizer,
};

use super::Sound;

/// Store a kit patch
///
/// A kit is basically an array of RowKit.
///
/// The rows order are visually reversed by the deluge. In the XML file, the rows
/// are logically ordered as we expect meaning the index increase as we add new row.
#[derive(Clone, Debug, PartialEq)]
pub struct Kit {
    pub rows: Vec<RowKit>,

    pub selected_drum_index: Option<u32>,

    pub lpf_mode: LpfMode,
    /// The current type of filter controled by the gold buttons
    pub current_filter_type: FilterType,
    /// The modulation FX global for the kit
    pub modulation_fx_type: ModulationFxType,
    /// The global delay
    pub delay: Delay,

    pub sidechain: Sidechain,

    /// The global low pass filter
    pub lpf: Lpf,

    /// The global high pass filter
    pub hpf: Hpf,

    pub equalizer: Equalizer,
}

impl Kit {
    pub fn new(rows: Vec<RowKit>) -> Self {
        let has_rows = rows.is_empty();

        Self {
            rows,
            lpf_mode: LpfMode::Lpf24,
            modulation_fx_type: ModulationFxType::Flanger,
            current_filter_type: FilterType::Lpf,
            selected_drum_index: if has_rows { None } else { Some(0) },
            delay: Delay::default(),
            sidechain: Sidechain::default(),
            lpf: Lpf::default(),
            hpf: Hpf::default(),
            equalizer: Equalizer::default(),
        }
    }

    pub fn add_row(&mut self, row: RowKit) -> usize {
        let index = self.rows.len();
        self.rows.push(row);

        index
    }

    pub fn add_sound_row(&mut self, sound: Sound) -> usize {
        self.add_row(RowKit::new_audio(sound, &format!("U{}", self.rows.len() + 1)))
    }

    pub fn add_sound_row_with_name(&mut self, sound: Sound, name: &str) -> usize {
        self.add_row(RowKit::new_audio(sound, name))
    }

    pub fn add_midi_row(&mut self, channel: MidiChannel, note: u8) -> usize {
        self.add_row(RowKit::new_midi(channel, note))
    }

    pub fn add_gate_row(&mut self, channel: CvGateChannel) -> usize {
        self.add_row(RowKit::new_cv_gate(channel))
    }
}

/// Default implementation for Kit
///
/// This implementation returns a Kit exactly like the Deluge would create it without any user changes.
impl Default for Kit {
    fn default() -> Self {
        let osc1 = Oscillator::new_sample(Sample::OneZone(SampleOneZone {
            file_path: String::new(),
            zone: Some(SampleZone {
                start: SamplePosition::new(0),
                end: SamplePosition::new(9999999),
                start_loop: None,
                end_loop: None,
            }),
        }));
        let mut osc2 = Oscillator::new_sample(Sample::OneZone(SampleOneZone {
            file_path: String::new(),
            zone: None,
        }));

        osc2.set_volume(0.into());

        let mut default_sound = Sound::new_substractive(osc1, osc2);

        default_sound.polyphonic = Polyphony::Auto;
        default_sound.mod_knobs[12].control_param = "pitch".to_string();

        Self::new(vec![RowKit::AudioOutput(AudioOutput::new(default_sound, "U1"))])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lpf {
    pub frequency: HexU50,
    pub resonance: HexU50,
}

impl Default for Lpf {
    fn default() -> Self {
        Self { frequency: 50.into(), resonance: 0.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hpf {
    pub frequency: HexU50,
    pub resonance: HexU50,
}

impl Default for Hpf {
    fn default() -> Self {
        Self { frequency: 0.into(), resonance: 0.into() }
    }
}

/// An output
///
/// There are 3 different types of physical outputs for the Deluge:
///  - audio
///  - MIDI
///  - CV gate
/// Each row in a Kit is an output and can be any of the 3.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum RowKit {
    AudioOutput(AudioOutput),
    MidiOutput(MidiOutput),
    CvGateOutput(CvGateOutput),
}

impl RowKit {
    pub fn new_audio(sound: Sound, name: &str) -> Self {
        RowKit::AudioOutput(AudioOutput::new(sound, name))
    }

    pub fn new_midi(channel: MidiChannel, note: u8) -> Self {
        RowKit::MidiOutput(MidiOutput { channel, note })
    }

    pub fn new_cv_gate(channel: CvGateChannel) -> Self {
        RowKit::CvGateOutput(CvGateOutput { channel })
    }
}

/// Audio output is a regular synth patch with a name.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioOutput {
    /// Sound is 320 bytes so I'm boxing it to reduce the size of AudioOutput on the stack.
    /// Box allocates his memory on the heap.
    pub sound: Box<Sound>,
    /// The displayed name
    pub name: String,
}

impl AudioOutput {
    pub fn new(sound: Sound, name: &str) -> Self {
        Self {
            sound: Box::new(sound),
            name: name.to_string(),
        }
    }
}

/// The MIDI output is a MIDI channel and a MIDI note.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MidiOutput {
    pub channel: MidiChannel,
    pub note: u8,
}

/// The CV Gate output is the CV Gate channel only
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CvGateOutput {
    pub channel: CvGateChannel,
}

impl CvGateOutput {
    pub fn new(channel: CvGateChannel) -> Self {
        Self { channel }
    }
}

#[cfg(test)]
mod tests {
    use crate::{load_kit, save_kit, Kit};
    use pretty_assertions::assert_eq;

    #[test]
    fn default_kit_test() {
        let default_kit = Kit::default();
        let expected_default_kit = load_kit(include_str!("data_tests/default/KIT Default Test.XML")).unwrap();

        assert_eq!(expected_default_kit, default_kit)
    }

    #[test]
    fn test_load_write_load_kit_community_patches_synth_hats() {
        let kit = load_kit(include_str!("data_tests/KITS/Synth Hats.XML")).unwrap();
        let xml = save_kit(&kit).unwrap();
        println!("{}", xml);
        let reloaded_kit = load_kit(&xml).unwrap();

        assert_eq!(reloaded_kit, kit);
    }
}
