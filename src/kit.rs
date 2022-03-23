use super::Sound;

#[derive(Derivative, Clone, Debug, Default)]
#[derivative(PartialEq)]
pub struct Kit {
    #[derivative(PartialEq = "ignore")]
    pub firmware_version: Option<String>,
    #[derivative(PartialEq = "ignore")]
    pub earliest_compatible_firmware: Option<String>,
    pub rows: Vec<SoundSource>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(enum_as_inner::EnumAsInner))]
pub enum SoundSource {
    /// Sound is 320 bytes so I'm boxing it to reduce the size of the enum.
    SoundOutput(SoundOutput),
    MidiOutput(MidiOutput),
    GateOutput(GateOutput),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundOutput {
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
