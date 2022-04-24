use crate::{
    values::{CvGateChannel, FilterType, HexU50, LpfMode, MidiChannel, Pan, Polyphony, SamplePath},
    Delay, Equalizer, Flanger, ModulationFx, Sample, SampleOneZone, SampleZone, Sidechain, Sound, SubtractiveOscillator,
};

mod row;

pub use row::{CvGateRow, MidiRow, RowKit, SoundRow};

/// Store a kit patch
///
/// A kit is basically an array of RowKit.
///
/// The rows order are visually reversed by the deluge. In the XML file, the rows
/// are logically ordered as we expect meaning the index increase as we add new row.
#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct Kit {
    #[builder(setter(each(name = "add_row")))]
    pub rows: Vec<RowKit>,

    pub selected_drum_index: Option<u32>,

    pub volume: HexU50,
    pub pan: Pan,
    pub reverb_amount: HexU50,
    pub lpf_mode: LpfMode,

    /// The current type of filter controled by the gold buttons
    pub current_filter_type: FilterType,

    pub bit_crush: HexU50,
    pub decimation: HexU50,
    pub stutter_rate: HexU50,

    /// The modulation FX global for the kit
    pub modulation_fx: ModulationFx,

    /// The global delay
    pub delay: Delay,

    /// The global sidechain
    pub sidechain: Sidechain,

    /// The global low pass filter
    pub lpf: Lpf,

    /// The global high pass filter
    pub hpf: Hpf,

    /// The global equalizer
    pub equalizer: Equalizer,
}

impl Kit {
    pub fn new(rows: Vec<RowKit>) -> Self {
        let has_rows = rows.is_empty();

        Self {
            rows,
            lpf_mode: LpfMode::Lpf24,
            modulation_fx: ModulationFx::Flanger(Flanger {
                rate: 19.into(),
                feedback: 0.into(),
            }),
            volume: 35.into(),
            pan: Pan::default(),
            reverb_amount: 0.into(),
            current_filter_type: FilterType::Lpf,
            bit_crush: 0.into(),
            decimation: 0.into(),
            stutter_rate: 25.into(),
            selected_drum_index: if has_rows { None } else { Some(0) },
            delay: Delay::default(),
            sidechain: Sidechain::default(),
            lpf: Lpf::default(),
            hpf: Hpf::default(),
            equalizer: Equalizer::default(),
        }
    }

    fn add_row(&mut self, row: RowKit) -> &mut RowKit {
        self.rows.push(row);

        self.rows.last_mut().unwrap()
    }

    pub fn add_sound_row(&mut self, sound: Sound) -> &mut Sound {
        self.add_named_sound(sound, &format!("U{}", self.rows.len() + 1))
    }

    /// Add a sound row with a custom name
    /// ```
    /// use deluge::{Kit, Sound, SamplePath};
    ///
    /// let mut kit = Kit::default();
    /// kit.add_named_sound(Sound::new_sample(SamplePath::new("sample.WAV").unwrap(), 0.into(), 999.into()), "My sample");
    /// ```
    pub fn add_named_sound(&mut self, sound: Sound, name: &str) -> &mut Sound {
        &mut self.add_row(RowKit::new_sound(sound, name)).as_sound_mut().unwrap().sound
    }

    /// Add a MIDI row
    /// ```
    /// use deluge::Kit;
    ///
    /// let mut kit = Kit::default();
    /// kit.add_midi_row(1.into(), 60);
    /// ```
    pub fn add_midi_row(&mut self, channel: MidiChannel, note: u8) {
        self.add_row(RowKit::new_midi(channel, note));
    }

    /// Add a CV gate row
    /// ```
    /// use deluge::Kit;
    ///
    /// let mut kit = Kit::default();
    /// kit.add_gate_row(1.into());
    /// ```
    pub fn add_gate_row(&mut self, channel: CvGateChannel) {
        self.add_row(RowKit::new_cv_gate(channel));
    }
}

/// Default implementation for Kit
///
/// This implementation returns a Kit exactly like the Deluge would create it without any user changes.
impl Default for Kit {
    fn default() -> Self {
        let osc1 = SubtractiveOscillator::new_sample(Sample::OneZone(SampleOneZone {
            file_path: SamplePath::default(),
            zone: Some(SampleZone {
                start: 0.into(),
                end: 9999999.into(),
                start_loop: None,
                end_loop: None,
            }),
        }));
        let osc2 = SubtractiveOscillator::new_sample(Sample::OneZone(SampleOneZone {
            file_path: SamplePath::default(),
            zone: None,
        }));

        let mut default_sound = Sound::new_substractive(osc1, osc2);

        default_sound.polyphonic = Polyphony::Auto;
        default_sound.mod_knobs[12].control_param = "pitch".to_string();

        Self::new(vec![RowKit::Sound(SoundRow::new(default_sound, "U1"))])
    }
}

impl KitBuilder {
    /// Add a sound row with a custom name
    /// ```
    /// use deluge::{Kit, Sound, KitBuilder, SamplePath};
    ///
    /// let mut kit = KitBuilder::default()
    ///     .add_sound_row(Sound::new_sample(SamplePath::new("sample.WAV").unwrap(), 0.into(), 999.into()))
    ///     .build()
    ///     .unwrap()
    ///     ;
    /// ```
    pub fn add_sound_row(&mut self, sound: Sound) -> &mut Self {
        self.add_named_sound_row(
            sound,
            &format!("U{}", self.rows.as_ref().map(|rows| rows.len()).unwrap_or_default() + 1),
        );

        self
    }

    pub fn add_named_sound_row(&mut self, sound: Sound, name: &str) -> &mut Self {
        self.add_row(RowKit::new_sound(sound, name))
    }

    pub fn add_midi_row(&mut self, channel: MidiChannel, note: u8) -> &mut Self {
        self.add_row(RowKit::new_midi(channel, note))
    }

    pub fn add_gate_row(&mut self, channel: CvGateChannel) -> &mut Self {
        self.add_row(RowKit::new_cv_gate(channel))
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct Lpf {
    pub frequency: HexU50,
    pub resonance: HexU50,
}

impl Default for Lpf {
    fn default() -> Self {
        Self {
            frequency: 50.into(),
            resonance: 0.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct Hpf {
    pub frequency: HexU50,
    pub resonance: HexU50,
}

impl Default for Hpf {
    fn default() -> Self {
        Self {
            frequency: 0.into(),
            resonance: 0.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{load_kit, save_kit, Kit};
    use pretty_assertions::assert_eq;

    #[test]
    fn default_kit_test() {
        let default_kit = Kit::default();
        let expected_default_kit = load_kit(include_str!("../data_tests/default/KIT Default Test.XML")).unwrap();

        assert_eq!(expected_default_kit, default_kit)
    }

    #[test]
    fn test_load_write_load_kit_community_patches_synth_hats() {
        let kit = load_kit(include_str!("../data_tests/KITS/Synth Hats.XML")).unwrap();
        let xml = save_kit(&kit).unwrap();
        let reloaded_kit = load_kit(&xml).unwrap();

        assert_eq!(reloaded_kit, kit);
    }
}
