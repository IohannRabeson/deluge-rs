use crate::Sound;

/// Default implementation for Kit
/// 
/// The default Synth is exactly like the Deluge would create it for a default synth patch without any user changes.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Synth {
    pub sound: Sound,
}
