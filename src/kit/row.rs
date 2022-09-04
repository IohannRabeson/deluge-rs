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
#[derive(Clone, Debug, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum RowKit {
    Sound(SoundRow),
    Midi(MidiRow),
    CvGate(CvGateRow),
}

impl RowKit {
    pub fn new_sound(sound: Sound, name: &str) -> Self {
        RowKit::Sound(SoundRow::new(sound, name))
    }

    pub fn new_midi(channel: MidiChannel, note: u8) -> Self {
        RowKit::Midi(MidiRow { channel, note })
    }

    pub fn new_cv_gate(channel: CvGateChannel) -> Self {
        RowKit::CvGate(CvGateRow { channel })
    }
}

/// Audio output is a regular synth patch with a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundRow {
    /// Sound is 320 bytes so I'm boxing it to reduce the size of AudioOutput on the stack.
    /// Box allocates his memory on the heap.
    pub sound: Box<Sound>,
    /// The displayed name
    pub name: String,
}

impl SoundRow {
    pub fn new(sound: Sound, name: &str) -> Self {
        Self {
            sound: Box::new(sound),
            name: name.to_string(),
        }
    }
}

/// The MIDI output is a MIDI channel and a MIDI note.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MidiRow {
    pub channel: MidiChannel,
    pub note: u8,
}

/// The CV Gate output is the CV Gate channel only
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CvGateRow {
    pub channel: CvGateChannel,
}

impl CvGateRow {
    pub fn new(channel: CvGateChannel) -> Self {
        Self { channel }
    }
}
