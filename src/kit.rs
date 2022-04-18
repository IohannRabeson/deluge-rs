use crate::{
    values::{HexU50, Polyphony},
    Oscillator, Sample, SampleOneZone, SamplePosition, SampleZone,
};

use super::Sound;

/// Store a kit patch
/// 
/// A kit is basically an array of SoundSource.
/// 
/// The rows order are visually reversed by the deluge. In the XML file, the rows
/// are logically ordered as we expect meaning the index increase as we add new row.
#[derive(Clone, Debug, PartialEq)]
pub struct Kit {
    pub rows: Vec<Output>,
}

impl Kit {
    pub fn new(rows: Vec<Output>) -> Self {
        Self { rows }
    }

    pub fn add_sound_row(&mut self, sound: Sound, name: &str) -> usize {
        let source = Output::AudioOutput(AudioOutput {
            name: name.to_string(),
            sound: Box::new(sound),
        });

        let index = self.rows.len();

        self.rows.push(source);

        index
    }

    pub fn add_midi_row(&mut self, channel: u8, note: u8) -> usize {
        let source = Output::MidiOutput(MidiOutput { channel, note });

        let index = self.rows.len();

        self.rows.push(source);

        index
    }

    pub fn add_gate_row(&mut self, channel: u8) -> usize {
        let source = Output::CvGateOutput(CvGateOutput { channel });

        let index = self.rows.len();

        self.rows.push(source);

        index
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

        osc2.set_volume(HexU50::new(0));

        let mut default_sound = Sound::new_substractive(osc1, osc2);

        default_sound.polyphonic = Polyphony::Auto;
        default_sound.mod_knobs[12].control_param = "pitch".to_string();

        Self::new(vec![Output::AudioOutput(AudioOutput::new(default_sound, "U1"))])
    }
}

/// An output
/// 
/// There are 3 different types of physical outputs for the Deluge:
///  - audio
///  - MIDI
///  - CV gate
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum Output {
    AudioOutput(AudioOutput),
    MidiOutput(MidiOutput),
    CvGateOutput(CvGateOutput),
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
    // TODO use Uint<1, 16, 1>;
    pub channel: u8,
    pub note: u8,
}

/// The CV Gate output is the CV Gate channel only
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CvGateOutput {
    // TODO use Uint8<1, 4, 1>;
    pub channel: u8,
}

#[cfg(test)]
mod tests {
    use crate::{load_kit, Kit};
    use pretty_assertions::assert_eq;

    #[test]
    fn default_kit_test() {
        let default_kit = Kit::default();
        let expected_default_kit = load_kit(include_str!("data_tests/default/KIT Default Test.XML")).unwrap();

        assert_eq!(expected_default_kit, default_kit)
    }
}
