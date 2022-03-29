use super::Sound;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Kit {
    pub rows: Vec<SoundSource>,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MidiOutput {
    pub channel: u8,
    pub note: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GateOutput {
    pub channel: u8,
}
