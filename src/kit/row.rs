use crate::{
    values::{CvGateChannel, MidiChannel},
    Sound,
};

/// A row in a kit
///
/// There are 3 different types of row for the Deluge:
///  - Deluge engine
///  - MIDI
///  - CV gate
/// Each row in a Kit is an output and can be any of the 3 types.
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
