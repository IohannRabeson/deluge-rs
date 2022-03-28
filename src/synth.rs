use crate::Sound;

#[derive(Derivative, Clone, Debug)]
#[derivative(PartialEq)]
pub struct Synth {
    pub sound: Sound,
}
