use crate::{
    values::{HexU50, Polyphony},
    Oscillator, Sample, SampleOneZone, SamplePosition, SampleZone,
};

use super::Sound;

#[derive(Clone, Debug, PartialEq)]
pub struct Kit {
    pub rows: Vec<SoundSource>,
}

impl Kit {
    pub fn new(rows: Vec<SoundSource>) -> Self {
        Self { rows }
    }

    pub fn add_sound_row(&mut self, sound: Sound, name: &str) -> usize {
        let source = SoundSource::SoundOutput(SoundOutput {
            name: name.to_string(),
            sound: Box::new(sound),
        });

        let index = self.rows.len();

        self.rows.push(source);

        index
    }

    pub fn add_midi_row(&mut self, channel: u8, note: u8) -> usize {
        let source = SoundSource::MidiOutput(MidiOutput { channel, note });

        let index = self.rows.len();

        self.rows.push(source);

        index
    }

    pub fn add_gate_row(&mut self, channel: u8) -> usize {
        let source = SoundSource::GateOutput(GateOutput { channel });

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

        Self::new(vec![SoundSource::SoundOutput(SoundOutput::new(default_sound, "U1"))])
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum SoundSource {
    SoundOutput(SoundOutput),
    MidiOutput(MidiOutput),
    GateOutput(GateOutput),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundOutput {
    /// Sound is 320 bytes so I'm boxing it to reduce the size of SoundSource.
    pub sound: Box<Sound>,
    pub name: String,
}

impl SoundOutput {
    pub fn new(sound: Sound, name: &str) -> Self {
        Self {
            sound: Box::new(sound),
            name: name.to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MidiOutput {
    pub channel: u8,
    pub note: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GateOutput {
    pub channel: u8,
}
