use crate::values::{FineTranspose, HexU50, OnOff, RetrigPhase, Transpose};

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct FmSynth {
    pub osc1: FmCarrier,
    pub osc2: FmCarrier,
    pub modulator1: FmModulator,
    pub modulator2: FmModulator,
    pub osc1_volume: HexU50,
    pub osc2_volume: HexU50,
    /// Parameter "Destination"
    /// If On modulator 2 modulates the modulator 1, otherwise it modulates the carrier 2.
    pub modulator2_to_modulator1: OnOff,
}

impl FmSynth {
    pub fn new(osc1: FmCarrier, osc2: FmCarrier) -> Self {
        Self {
            osc1,
            osc2,
            modulator1: FmModulator::default(),
            modulator2: FmModulator::default(),
            modulator2_to_modulator1: OnOff::Off,
            osc1_volume: 50.into(),
            osc2_volume: 39.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct FmCarrier {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub feedback: HexU50,
}

impl Default for FmCarrier {
    fn default() -> Self {
        Self {
            transpose: Default::default(),
            fine_transpose: Default::default(),
            retrig_phase: Default::default(),
            feedback: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
pub struct FmModulator {
    pub transpose: Transpose,
    pub fine_transpose: FineTranspose,
    pub retrig_phase: RetrigPhase,
    pub amount: HexU50,
    pub feedback: HexU50,
}

impl Default for FmModulator {
    fn default() -> Self {
        Self {
            transpose: Default::default(),
            fine_transpose: Default::default(),
            retrig_phase: RetrigPhase::Off,
            amount: 0.into(),
            feedback: 0.into(),
        }
    }
}
