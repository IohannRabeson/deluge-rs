use crate::Sound;

#[derive(Derivative, Clone, Debug)]
#[derivative(PartialEq)]
pub struct Synth {
    /// Not specified when loading a sound in a kit
    #[derivative(PartialEq = "ignore")]
    pub firmware_version: Option<String>,
    /// Not specified when loading a sound in a kit
    #[derivative(PartialEq = "ignore")]
    pub earliest_compatible_firmware: Option<String>,

    pub sound: Sound,
}
