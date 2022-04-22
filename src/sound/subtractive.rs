use crate::{
    values::{FineTranspose, HexU50, LpfMode, OnOff, OscType, RetrigPhase, Transpose},
    Oscillator, WaveformOscillator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SubtractiveGenerator {
    pub osc1: Oscillator,
    pub osc2: Oscillator,
    pub osc2_sync: OnOff,
    pub osc1_volume: HexU50,
    pub osc2_volume: HexU50,
    pub noise: HexU50,
    pub lpf_mode: LpfMode,
    pub lpf_frequency: HexU50,
    pub lpf_resonance: HexU50,
    pub hpf_frequency: HexU50,
    pub hpf_resonance: HexU50,
}

impl SubtractiveGenerator {
    pub fn new(osc1: Oscillator, osc2: Oscillator) -> Self {
        Self {
            osc1,
            osc2,
            ..Default::default()
        }
    }
}

impl Default for SubtractiveGenerator {
    fn default() -> Self {
        let osc1 = Oscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            pulse_width: 25.into(),
        });

        let osc2 = Oscillator::Waveform(WaveformOscillator {
            osc_type: OscType::Square,
            transpose: Transpose::default(),
            fine_transpose: FineTranspose::default(),
            retrig_phase: RetrigPhase::Off,
            pulse_width: 25.into(),
        });

        Self {
            osc1,
            osc2,
            osc2_sync: OnOff::Off,
            osc1_volume: 50.into(),
            osc2_volume: 0.into(),
            noise: 0.into(),
            lpf_mode: LpfMode::Lpf24,
            lpf_frequency: 50.into(),
            lpf_resonance: 0.into(),
            hpf_frequency: 0.into(),
            hpf_resonance: 0.into(),
        }
    }
}
