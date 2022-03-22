use crate::Sound;

#[derive(Derivative, Clone, Debug)]
#[derivative(PartialEq)]
pub struct Synth {
    #[derivative(PartialEq = "ignore")]
    pub firmware_version: Option<String>,
    
    #[derivative(PartialEq = "ignore")]
    pub earliest_compatible_firmware: Option<String>,

    pub sound: Sound,
}
