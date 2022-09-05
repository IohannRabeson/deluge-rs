use crate::{
    values::{FineTranspose, HexU50, OnOff, OscType, RetrigPhase, Transpose},
    WaveformOscillator,
};

#[derive(Clone, Debug, PartialEq, Eq, derive_builder::Builder)]
#[builder(default)]
pub struct RingModSynth {
    pub osc1: WaveformOscillator,
    pub osc2: WaveformOscillator,
    pub osc2_sync: OnOff,
    pub noise: HexU50,
}

impl RingModSynth {
    pub fn new(osc1: WaveformOscillator, osc2: WaveformOscillator) -> Self {
        Self {
            osc1,
            osc2,
            ..Default::default()
        }
    }
}

impl Default for RingModSynth {
    fn default() -> Self {
        let osc1 = WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            pulse_width: 25.into(),
        };

        let osc2 = WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            pulse_width: 25.into(),
        };

        Self {
            osc1,
            osc2,
            osc2_sync: OnOff::Off,
            noise: 0.into(),
        }
    }
}
