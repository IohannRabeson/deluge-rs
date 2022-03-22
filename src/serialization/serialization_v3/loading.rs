use crate::{
    serialization::{
        default_params::{DefaultParams, TwinSelector},
        keys,
        serialization_common::{convert_milliseconds_to_samples, parse_u8},
        xml,
    },
    values::*,
    Arpeggiator, Chorus, Delay, Distorsion, Envelope, Equalizer, Error, Flanger, FmCarrier, FmGenerator, FmModulator, GateOutput,
    Kit, Lfo1, Lfo2, MidiOutput, ModKnob, ModulationFx, Oscillator, PatchCable, Phaser, RingModGenerator, Sample, SampleOneZone,
    SampleOscillator, SamplePosition, SampleRange, SampleZone, Sidechain, Sound, SoundGenerator, SoundSource,
    SubtractiveGenerator, Unison, WaveformOscillator,
};

use xmltree::Element;

/// Load a deluge synth XML file
pub fn load_synth_nodes(root_nodes: &[Element]) -> Result<Sound, Error> {
    let sound_node = xml::get_element(root_nodes, keys::SOUND)?;
    let firmware_version = xml::parse_opt_attribute(sound_node, keys::FIRMWARE_VERSION)?;
    let earliest_compatible_firmware = xml::parse_opt_attribute(sound_node, keys::EARLIEST_COMPATIBLE_FIRMWARE)?;
    let mut sound = load_sound(sound_node)?;

    sound.firmware_version = firmware_version;
    sound.earliest_compatible_firmware = earliest_compatible_firmware;

    Ok(sound)
}

pub fn load_kit(xml: &str) -> Result<Kit, Error> {
    let root_nodes: Vec<Element> = xml::load_xml(xml)?;
    let kit_node = xml::get_element(&root_nodes, keys::KIT)?;
    let sound_sources_node = xml::get_children_element(kit_node, keys::SOUND_SOURCES)?;
    let firmware_version = xml::parse_opt_attribute(kit_node, keys::FIRMWARE_VERSION)?;
    let earliest_compatible_firmware = xml::parse_opt_attribute(kit_node, keys::EARLIEST_COMPATIBLE_FIRMWARE)?;
    let sources: Vec<Result<SoundSource, Error>> = sound_sources_node
        .children
        .iter()
        .filter_map(xml::keep_element_only)
        .map(load_sound_source)
        .collect();

    if let Some(result_with_error) = sources.iter().find(|s| s.is_err()) {
        return Err(result_with_error.as_ref().unwrap_err().clone());
    }

    return Ok(Kit {
        firmware_version,
        earliest_compatible_firmware,
        rows: sources.iter().flatten().cloned().collect::<Vec<SoundSource>>(),
    });
}

/// Load a "sound" node.
///
/// A sound can be a whole synth patch, or row in a kit patch.
/// At least I hope. I think a Sound is stored by a Row kit because a
/// row kit is a sound with few additional attributes like the name of the row.
/// I think the class structure in the deluge implementation looks like:
/// class Sound
/// class RowKit(Sound, Name, OtherAdditionalInfosByRow)
fn load_sound(root: &Element) -> Result<Sound, Error> {
    let sound_type = xml::parse_attribute::<SoundType>(root, keys::MODE)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    let generator = match sound_type {
        SoundType::Subtractive => load_subtractive_sound(root)?,
        SoundType::Fm => load_fm_sound(root)?,
        SoundType::RingMod => load_ringmode_sound(root)?,
        _ => return Err(Error::UnsupportedSoundType),
    };

    Ok(Sound {
        name: xml::parse_opt_attribute(root, keys::NAME)?.unwrap_or_default(),
        firmware_version: xml::parse_opt_attribute(root, keys::FIRMWARE_VERSION)?,
        earliest_compatible_firmware: xml::parse_opt_attribute(root, keys::EARLIEST_COMPATIBLE_FIRMWARE)?,

        polyphonic: xml::parse_attribute(root, keys::POLYPHONIC)?,
        voice_priority: xml::parse_attribute(root, keys::VOICE_PRIORITY)?,
        volume: xml::parse_attribute(default_params_node, keys::VOLUME)?,
        reverb_amount: xml::parse_attribute(default_params_node, keys::REVERB_AMOUNT)?,
        stutter_rate: xml::parse_attribute(default_params_node, keys::STUTTER_RATE)?,
        pan: xml::parse_attribute(default_params_node, keys::PAN)?,
        portamento: xml::parse_attribute(default_params_node, keys::PORTAMENTO)?,
        sidechain_send: xml::parse_opt_attribute(root, keys::SIDECHAIN_SEND)?,
        generator,
        envelope1: load_envelope(xml::get_children_element(default_params_node, keys::ENVELOPE1)?)?,
        envelope2: load_envelope(xml::get_children_element(default_params_node, keys::ENVELOPE2)?)?,
        lfo1: load_lfo1(xml::get_children_element(root, keys::LFO1)?, default_params_node)?,
        lfo2: load_lfo2(xml::get_children_element(root, keys::LFO2)?, default_params_node)?,
        unison: load_unison(xml::get_children_element(root, keys::UNISON)?)?,
        arpeggiator: load_arpeggiator(xml::get_children_element(root, keys::ARPEGGIATOR)?, default_params_node)?,
        delay: load_delay(xml::get_children_element(root, keys::DELAY)?, default_params_node)?,
        distorsion: load_distorsion(root, default_params_node)?,
        equalizer: load_equalizer(xml::get_children_element(default_params_node, keys::EQUALIZER)?)?,
        modulation_fx: load_modulation_fx(root)?,
        sidechain: load_sidechain(xml::get_children_element(root, keys::COMPRESSOR)?, default_params_node)?,
        cables: load_patch_cables(xml::get_children_element(default_params_node, keys::PATCH_CABLES)?)?,
        mod_knobs: load_mod_knobs(xml::get_children_element(root, keys::MOD_KNOBS)?)?,
    })
}

fn load_subtractive_sound(root: &Element) -> Result<SoundGenerator, Error> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    Ok(SoundGenerator::Subtractive(SubtractiveGenerator {
        osc1: load_oscillator(osc1_node, &DefaultParams::new(TwinSelector::A, default_params_node))?,
        osc2: load_oscillator(osc2_node, &DefaultParams::new(TwinSelector::B, default_params_node))?,
        osc2_sync: xml::parse_opt_attribute(osc2_node, keys::OSCILLATOR_SYNC)?.unwrap_or(OnOff::Off),
        noise: xml::parse_attribute(default_params_node, keys::NOISE_VOLUME)?,
        lpf_mode: xml::parse_attribute(root, keys::LPF_MODE)?,
        lpf_frequency: xml::parse_attribute(default_params_node, keys::LPF_FREQUENCY)?,
        lpf_resonance: xml::parse_attribute(default_params_node, keys::LPF_RESONANCE)?,
        hpf_frequency: xml::parse_attribute(default_params_node, keys::HPF_FREQUENCY)?,
        hpf_resonance: xml::parse_attribute(default_params_node, keys::HPF_RESONANCE)?,
    }))
}

fn load_ringmode_sound(root: &Element) -> Result<SoundGenerator, Error> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    Ok(SoundGenerator::RingMod(RingModGenerator {
        osc1: load_oscillator(osc1_node, &DefaultParams::new(TwinSelector::A, default_params_node))?,
        osc2: load_oscillator(osc2_node, &DefaultParams::new(TwinSelector::B, default_params_node))?,
        osc2_sync: xml::parse_opt_attribute::<OnOff>(osc2_node, keys::OSCILLATOR_SYNC)?.unwrap_or(OnOff::Off),
    }))
}

fn load_fm_sound(root: &Element) -> Result<SoundGenerator, Error> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let mod1_node = xml::get_children_element(root, keys::FM_MODULATOR1)?;
    let mod2_node = xml::get_children_element(root, keys::FM_MODULATOR2)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;
    let params_a = &DefaultParams::new(TwinSelector::A, default_params_node);
    let params_b = &DefaultParams::new(TwinSelector::B, default_params_node);

    Ok(SoundGenerator::Fm(FmGenerator {
        osc1: load_carrier(osc1_node, params_a)?,
        osc2: load_carrier(osc2_node, params_b)?,
        modulator1: load_fm_modulation(mod1_node, params_a)?,
        modulator2: load_fm_modulation(mod2_node, params_b)?,
        modulator2_to_modulator1: xml::parse_attribute(mod2_node, keys::FM_MOD1_TO_MOD2)?,
    }))
}

fn load_oscillator(root: &Element, params: &DefaultParams) -> Result<Oscillator, Error> {
    let osc_type = xml::parse_attribute(root, keys::TYPE)?;

    match osc_type {
        OscType::Sample => load_sample_oscillator(root, params),
        OscType::AnalogSaw => load_waveform_oscillator(osc_type, root, params),
        OscType::AnalogSquare => load_waveform_oscillator(osc_type, root, params),
        OscType::Saw => load_waveform_oscillator(osc_type, root, params),
        OscType::Sine => load_waveform_oscillator(osc_type, root, params),
        OscType::Square => load_waveform_oscillator(osc_type, root, params),
        OscType::Triangle => load_waveform_oscillator(osc_type, root, params),
    }
}

fn load_carrier(root: &Element, params: &DefaultParams) -> Result<FmCarrier, Error> {
    Ok(FmCarrier {
        transpose: xml::parse_attribute(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_attribute(root, keys::CENTS)?,
        retrig_phase: xml::parse_attribute(root, keys::RETRIG_PHASE)?,
        volume: params.parse_twin_attribute(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B)?,
        feedback: params.parse_twin_attribute(keys::FEEDBACK_CARRIER1, keys::FEEDBACK_CARRIER2)?,
    })
}

fn load_fm_modulation(root: &Element, params: &DefaultParams) -> Result<FmModulator, Error> {
    Ok(FmModulator {
        transpose: xml::parse_attribute(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_attribute(root, keys::CENTS)?,
        retrig_phase: xml::parse_attribute(root, keys::RETRIG_PHASE)?,
        amount: params.parse_twin_attribute(keys::AMOUNT_MODULATOR1, keys::AMOUNT_MODULATOR2)?,
        feedback: params.parse_twin_attribute(keys::FEEDBACK_MODULATOR1, keys::FEEDBACK_MODULATOR2)?,
    })
}

fn load_sample_oscillator(root: &Element, params: &DefaultParams) -> Result<Oscillator, Error> {
    Ok(Oscillator::Sample(SampleOscillator {
        transpose: xml::parse_opt_attribute(root, keys::TRANSPOSE)?.unwrap_or_default(),
        fine_transpose: xml::parse_opt_attribute(root, keys::CENTS)?.unwrap_or_default(),
        reversed: xml::parse_attribute(root, keys::REVERSED)?,
        mode: xml::parse_attribute(root, keys::LOOP_MODE)?,
        pitch_speed: xml::parse_attribute(root, keys::TIME_STRETCH_ENABLE)?,
        time_stretch_amount: xml::parse_attribute(root, keys::TIME_STRETCH_AMOUNT)?,
        sample: load_sample(root)?,
        linear_interpolation: xml::parse_opt_attribute(root, keys::LINEAR_INTERPOLATION)?.unwrap_or_default(),
        volume: params.parse_twin_attribute(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B)?,
    }))
}

fn load_sample(root: &Element) -> Result<Sample, Error> {
    Ok(
        if let Some(sample_ranges_node) = xml::get_opt_children_element(root, keys::SAMPLE_RANGES) {
            let mut ranges: Vec<SampleRange> = Vec::new();
            let sample_range_nodes = xml::get_all_children_element_with_name(sample_ranges_node, keys::SAMPLE_RANGE);

            for sample_range_node in sample_range_nodes {
                let zone_node = xml::get_children_element(sample_range_node, keys::ZONE)?;
                let range = SampleRange {
                    range_top_note: xml::parse_opt_attribute(sample_range_node, keys::SAMPLE_RANGE_TOP_NOTE)?,
                    file_path: xml::parse_attribute(sample_range_node, keys::FILE_NAME)?,
                    transpose: xml::parse_opt_attribute(sample_range_node, keys::TRANSPOSE)?.unwrap_or_default(),
                    fine_transpose: xml::parse_opt_attribute(sample_range_node, keys::CENTS)?.unwrap_or_default(),
                    zone: parse_sample_zone(zone_node)?,
                };

                ranges.push(range);
            }

            Sample::SampleRanges(ranges)
        } else if let Some(sample_zone_node) = xml::get_opt_children_element(root, "zone") {
            Sample::OneZone(SampleOneZone {
                file_path: xml::parse_opt_attribute(root, keys::FILE_NAME)?.unwrap_or_default(),
                zone: Some(parse_sample_zone(sample_zone_node)?),
            })
        } else {
            Sample::OneZone(SampleOneZone {
                file_path: xml::parse_opt_attribute(root, keys::FILE_NAME)?.unwrap_or_default(),
                zone: None,
            })
        },
    )
}

/// Parse a sample zone
///
/// The root element must be a "zone" node.
/// We try to get start and end positions as samples if possible, and as milliseconds if forced.
/// If both are missing then SamplePosition(0) is assigned.
fn parse_sample_zone(root: &Element) -> Result<SampleZone, Error> {
    let start = SamplePosition::new(match xml::parse_opt_attribute::<u64>(root, keys::START_SAMPLES_POS)? {
        Some(samples) => samples,
        None => xml::parse_opt_attribute::<u64>(root, keys::START_MILLISECONDS_POS)?
            .map(convert_milliseconds_to_samples)
            .unwrap_or_default(),
    });

    let end = SamplePosition::new(match xml::parse_opt_attribute::<u64>(root, keys::END_SAMPLES_POS)? {
        Some(samples) => samples,
        None => xml::parse_opt_attribute::<u64>(root, keys::END_MILLISECONDS_POS)?
            .map(convert_milliseconds_to_samples)
            .unwrap_or_default(),
    });

    let start_loop = xml::parse_opt_attribute::<u64>(root, keys::START_LOOP_SAMPLES_POS)?.map(SamplePosition::new);

    let end_loop = xml::parse_opt_attribute::<u64>(root, keys::END_LOOP_SAMPLES_POS)?.map(SamplePosition::new);

    Ok(SampleZone {
        start,
        end,
        start_loop,
        end_loop,
    })
}

fn load_waveform_oscillator(osc_type: OscType, root: &Element, params: &DefaultParams) -> Result<Oscillator, Error> {
    Ok(Oscillator::Waveform(WaveformOscillator {
        osc_type,
        transpose: xml::parse_attribute(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_attribute(root, keys::CENTS)?,
        retrig_phase: xml::parse_attribute(root, keys::RETRIG_PHASE)?,
        volume: params.parse_twin_attribute(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B)?,
        pulse_width: params.parse_twin_attribute(keys::PULSE_WIDTH_OSC_A, keys::PULSE_WIDTH_OSC_B)?,
    }))
}

fn load_midi_output(root: &Element) -> Result<MidiOutput, Error> {
    let channel = xml::get_attribute(root, keys::CHANNEL).and_then(parse_u8)?;
    let note = xml::get_attribute(root, keys::NOTE).and_then(parse_u8)?;

    Ok(MidiOutput { channel, note })
}

fn load_gate_output(root: &Element) -> Result<GateOutput, Error> {
    xml::get_attribute(root, keys::CHANNEL)
        .and_then(parse_u8)
        .map(|channel| GateOutput { channel })
}

fn load_sound_source(root: &Element) -> Result<SoundSource, Error> {
    Ok(match root.name.as_str() {
        keys::SOUND => SoundSource::Sound(Box::new(load_sound(root)?)),
        keys::MIDI_OUTPUT => SoundSource::MidiOutput(load_midi_output(root)?),
        keys::GATE_OUTPUT => SoundSource::GateOutput(load_gate_output(root)?),
        _ => return Err(Error::UnsupportedSoundSource(root.name.clone())),
    })
}

fn load_envelope(root: &Element) -> Result<Envelope, Error> {
    Ok(Envelope {
        attack: xml::parse_attribute(root, keys::ENV_ATTACK)?,
        decay: xml::parse_attribute(root, keys::ENV_DECAY)?,
        sustain: xml::parse_attribute(root, keys::ENV_SUSTAIN)?,
        release: xml::parse_attribute(root, keys::ENV_RELEASE)?,
    })
}

fn load_lfo1(root: &Element, default_params_node: &Element) -> Result<Lfo1, Error> {
    Ok(Lfo1 {
        shape: xml::parse_attribute(root, keys::LFO_SHAPE)?,
        sync_level: xml::parse_attribute(root, keys::SYNC_LEVEL)?,
        rate: xml::parse_attribute(default_params_node, keys::LFO1_RATE)?,
    })
}

fn load_lfo2(root: &Element, default_params_node: &Element) -> Result<Lfo2, Error> {
    Ok(Lfo2 {
        shape: xml::parse_attribute(root, keys::LFO_SHAPE)?,
        rate: xml::parse_attribute(default_params_node, keys::LFO2_RATE)?,
    })
}

fn load_unison(root: &Element) -> Result<Unison, Error> {
    Ok(Unison {
        voice_count: xml::parse_attribute(root, keys::UNISON_VOICE_COUNT)?,
        detune: xml::parse_attribute(root, keys::UNISON_DETUNE)?,
    })
}

fn load_delay(root: &Element, default_params_node: &Element) -> Result<Delay, Error> {
    Ok(Delay {
        ping_pong: xml::parse_attribute(root, keys::PING_PONG)?,
        analog: xml::parse_attribute(root, keys::ANALOG)?,
        sync_level: xml::parse_attribute(root, keys::SYNC_LEVEL)?,
        amount: xml::parse_attribute(default_params_node, keys::DELAY_FEEDBACK)?,
        rate: xml::parse_attribute(default_params_node, keys::DELAY_RATE)?,
    })
}

fn load_arpeggiator(root: &Element, default_params_node: &Element) -> Result<Arpeggiator, Error> {
    Ok(Arpeggiator {
        mode: xml::parse_attribute(root, keys::ARPEGGIATOR_MODE)?,
        sync_level: xml::parse_attribute(root, keys::SYNC_LEVEL)?,
        octaves_count: xml::parse_attribute(root, keys::ARPEGGIATOR_OCTAVE_COUNT)?,
        rate: xml::parse_attribute(default_params_node, keys::ARPEGGIATOR_RATE)?,
        gate: xml::parse_attribute(default_params_node, keys::ARPEGGIATOR_GATE)?,
    })
}

fn load_distorsion(root: &Element, default_params_node: &Element) -> Result<Distorsion, Error> {
    Ok(Distorsion {
        saturation: xml::parse_opt_attribute(root, keys::CLIPPING_AMOUNT)?.unwrap_or_default(),
        bit_crush: xml::parse_attribute(default_params_node, keys::BIT_CRUSH)?,
        decimation: xml::parse_attribute(default_params_node, keys::DECIMATION)?,
    })
}

fn load_equalizer(root: &Element) -> Result<Equalizer, Error> {
    Ok(Equalizer {
        bass_level: xml::parse_attribute(root, keys::EQ_BASS)?,
        bass_frequency: xml::parse_attribute(root, keys::EQ_BASS_FREQUENCY)?,
        treble_level: xml::parse_attribute(root, keys::EQ_TREBLE)?,
        treble_frequency: xml::parse_attribute(root, keys::EQ_TREBLE_FREQUENCY)?,
    })
}

fn load_modulation_fx(root: &Element) -> Result<ModulationFx, Error> {
    let modulation_fx_type = xml::parse_attribute(root, keys::MOD_FX_TYPE)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    Ok(match modulation_fx_type {
        keys::MODULATION_FX_OFF => ModulationFx::Off,
        keys::MODULATION_FX_FLANGER => ModulationFx::Flanger(load_modulation_fx_flanger(default_params_node)?),
        keys::MODULATION_FX_CHORUS => ModulationFx::Chorus(load_modulation_fx_chorus(default_params_node)?),
        keys::MODULATION_FX_PHASER => ModulationFx::Phaser(load_modulation_fx_phaser(default_params_node)?),
        &_ => return Err(Error::UnsupportedModulationFx(modulation_fx_type.to_owned())),
    })
}

fn load_modulation_fx_flanger(default_params_node: &Element) -> Result<Flanger, Error> {
    Ok(Flanger {
        rate: xml::parse_attribute(default_params_node, keys::MODULATION_FX_RATE)?,
        feedback: xml::parse_attribute(default_params_node, keys::MODULATION_FX_FEEDBACK)?,
    })
}

fn load_modulation_fx_chorus(default_params_node: &Element) -> Result<Chorus, Error> {
    Ok(Chorus {
        rate: xml::parse_attribute(default_params_node, keys::MODULATION_FX_RATE)?,
        depth: xml::parse_attribute(default_params_node, keys::MODULATION_FX_DEPTH)?,
        offset: xml::parse_attribute(default_params_node, keys::MODULATION_FX_OFFSET)?,
    })
}

fn load_modulation_fx_phaser(default_params_node: &Element) -> Result<Phaser, Error> {
    Ok(Phaser {
        rate: xml::parse_attribute(default_params_node, keys::MODULATION_FX_RATE)?,
        depth: xml::parse_attribute(default_params_node, keys::MODULATION_FX_DEPTH)?,
        feedback: xml::parse_attribute(default_params_node, keys::MODULATION_FX_FEEDBACK)?,
    })
}

fn load_patch_cables(root: &Element) -> Result<Vec<PatchCable>, Error> {
    let cables = xml::get_all_children_element_with_name(root, keys::PATCH_CABLE);
    let mut patch_cables = Vec::new();

    for cable in cables {
        patch_cables.push(load_patch_cable(cable)?);
    }

    Ok(patch_cables)
}

fn load_mod_knob(element: &Element) -> Result<ModKnob, Error> {
    Ok(ModKnob {
        control_param: xml::parse_attribute(element, keys::MOD_KNOB_CONTROL_PARAM)?,
        patch_amount_from_source: xml::parse_opt_attribute(element, keys::MOD_KNOB_PATCH_AMOUNT_FROM_SOURCE)?,
    })
}

fn load_mod_knobs(root: &Element) -> Result<Vec<ModKnob>, Error> {
    let mod_knob_nodes = xml::get_all_children_element_with_name(root, keys::MOD_KNOB);
    let mut mod_knobs = Vec::new();

    for mod_knob_node in mod_knob_nodes {
        mod_knobs.push(load_mod_knob(mod_knob_node)?);
    }

    Ok(mod_knobs)
}

fn load_patch_cable(root: &Element) -> Result<PatchCable, Error> {
    Ok(PatchCable {
        source: xml::parse_attribute(root, keys::PATCH_CABLE_SOURCE)?,
        destination: xml::parse_attribute(root, keys::PATCH_CABLE_DESTINATION)?,
        amount: xml::parse_attribute(root, keys::PATCH_CABLE_AMOUNT)?,
    })
}

fn load_sidechain(root: &Element, default_params_node: &Element) -> Result<Sidechain, Error> {
    Ok(Sidechain {
        attack: xml::parse_attribute(root, keys::COMPRESSOR_ATTACK)?,
        release: xml::parse_attribute(root, keys::COMPRESSOR_RELEASE)?,
        shape: xml::parse_attribute(default_params_node, keys::COMPRESSOR_SHAPE)?,
        sync: xml::parse_attribute(root, keys::COMPRESSOR_SYNCLEVEL)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_valid_kit_xml() {
        assert!(load_kit(include_str!("../../data_tests/KITS/KIT057.XML")).is_ok());
    }

    #[test]
    fn load_valid_kit_xml_and_check_sounds_only() {
        let kit = load_kit(include_str!("../../data_tests/KITS/KIT_TEST_SOUNDS_ONLY.XML")).unwrap();

        assert_eq!(&kit.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&kit.earliest_compatible_firmware.unwrap(), "3.1.0-beta");
        assert_eq!(kit.rows.len(), 7);
    }

    #[test]
    fn load_valid_kit_xml_and_check_sounds_midi_and_gate() {
        let kit = load_kit(include_str!("../../data_tests/KITS/KIT_TEST_SOUNDS_MIDI_GATE.XML")).unwrap();

        assert_eq!(&kit.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&kit.earliest_compatible_firmware.unwrap(), "3.1.0-beta");
        assert_eq!(kit.rows.len(), 9);
        assert_eq!(kit.rows[0], SoundSource::MidiOutput(MidiOutput { channel: 1, note: 63 }));
        assert_eq!(kit.rows[1], SoundSource::GateOutput(GateOutput { channel: 3 }));
    }

    #[test]
    fn load_kit_check_row_name() {
        let kit = &load_kit(include_str!("../../data_tests/KITS/KIT057.XML")).unwrap();
        let expected = vec![
            "halftime_goodie",
            "halftime_goodie2",
            "halftime_goodie3",
            "halftime_goodie4",
            "halftime_goodie5",
            "halftime_goodie6",
            "halftime_goodie7",
        ];
        assert_eq!(kit.rows.len(), 7);

        for i in 0..kit.rows.len() {
            assert_eq!(kit.rows[i].as_sound().unwrap().name, expected[i]);
        }
    }

    #[test]
    fn load_valid_sound_subtractive() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT184.XML")).unwrap();
        let sound = load_synth_nodes(&xml_elements).unwrap();

        assert_eq!(&sound.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&sound.earliest_compatible_firmware.unwrap(), "3.1.0-beta");

        assert_eq!(sound.voice_priority, VoicePriority::Medium);
        assert_eq!(sound.polyphonic, Polyphony::Poly);
        assert_eq!(sound.volume, HexU50::parse("0x4CCCCCA8").unwrap());
        assert_eq!(sound.pan, Pan::parse("0x00000000").unwrap());
        assert_eq!(sound.portamento, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.modulation_fx, ModulationFx::Off);

        assert_eq!(sound.distorsion.saturation, ClippingAmount::new(4));
        assert_eq!(sound.distorsion.bit_crush, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.distorsion.decimation, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.lfo1.rate, HexU50::parse("0x1999997E").unwrap());
        assert_eq!(sound.lfo1.shape, LfoShape::Triangle);
        assert_eq!(sound.lfo1.sync_level, SyncLevel::Off);
        assert_eq!(sound.lfo2.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.lfo2.shape, LfoShape::Triangle);

        assert_eq!(sound.envelope1.attack, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope1.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope1.sustain, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(sound.envelope1.release, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.envelope2.attack, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.sustain, HexU50::parse("0xFFFFFFE9").unwrap());
        assert_eq!(sound.envelope2.release, HexU50::parse("0xE6666654").unwrap());

        assert_eq!(sound.unison.voice_count, UnisonVoiceCount::new(1));
        assert_eq!(sound.unison.detune, UnisonDetune::new(8));

        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Sixteenth);
        assert_eq!(sound.delay.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.sidechain.sync, SyncLevel::Sixteenth);
        assert_eq!(sound.sidechain.attack, AttackSidechain::try_from(327244).unwrap());
        assert_eq!(sound.sidechain.release, ReleaseSidechain::try_from(936).unwrap());

        assert_eq!(sound.equalizer.bass_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.bass_frequency, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_frequency, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.arpeggiator.mode, ArpeggiatorMode::Off);
        assert_eq!(sound.arpeggiator.octaves_count, OctavesCount::new(2));
        assert_eq!(sound.arpeggiator.gate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.arpeggiator.rate, HexU50::parse("0x00000000").unwrap());

        let generator = sound.generator.as_subtractive().unwrap();

        assert_eq!(generator.lpf_mode, LpfMode::Lpf24);
        assert_eq!(generator.lpf_frequency, HexU50::parse("0x147AE12D").unwrap());
        assert_eq!(generator.lpf_resonance, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.hpf_frequency, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.hpf_resonance, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.osc2_sync, OnOff::Off);

        let waveform = generator.osc1.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Square);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(0));
        assert_eq!(waveform.retrig_phase, RetrigPhase::default());
        assert_eq!(waveform.volume, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(waveform.pulse_width, HexU50::parse("0x00000000").unwrap());

        assert_eq!(1, sound.cables.len());
    }

    #[test]
    fn load_valid_sound_fm() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT176.XML")).unwrap();
        let sound = load_synth_nodes(&xml_elements).unwrap();

        assert_eq!(&sound.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&sound.earliest_compatible_firmware.unwrap(), "3.1.0-beta");

        assert_eq!(sound.voice_priority, VoicePriority::Medium);
        assert_eq!(sound.polyphonic, Polyphony::Poly);
        assert_eq!(sound.volume, HexU50::parse("0x1E000000").unwrap());
        assert_eq!(sound.pan, Pan::parse("0x00000000").unwrap());
        assert_eq!(sound.portamento, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.modulation_fx, ModulationFx::Off);

        assert_eq!(sound.distorsion.saturation, ClippingAmount::new(2));
        assert_eq!(sound.distorsion.bit_crush, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.distorsion.decimation, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.lfo1.rate, HexU50::parse("0xBD70A3CC").unwrap());
        assert_eq!(sound.lfo1.shape, LfoShape::Triangle);
        assert_eq!(sound.lfo1.sync_level, SyncLevel::Off);
        assert_eq!(sound.lfo2.rate, HexU50::parse("0xCCCCCCBF").unwrap());
        assert_eq!(sound.lfo2.shape, LfoShape::Triangle);

        assert_eq!(sound.unison.voice_count, UnisonVoiceCount::new(1));
        assert_eq!(sound.unison.detune, UnisonDetune::new(8));

        assert_eq!(sound.envelope1.attack, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope1.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope1.sustain, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(sound.envelope1.release, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope2.attack, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.sustain, HexU50::parse("0xFFFFFFE9").unwrap());
        assert_eq!(sound.envelope2.release, HexU50::parse("0xE6666654").unwrap());

        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Sixteenth);
        assert_eq!(sound.delay.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());

        let generator = sound.generator.as_fm().unwrap();

        assert_eq!(generator.osc1.transpose, Transpose::new(0));
        assert_eq!(generator.osc1.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.osc1.retrig_phase, RetrigPhase::default());
        assert_eq!(generator.osc1.volume, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(generator.osc1.feedback, HexU50::parse("0xCCCCCCBF").unwrap());

        assert_eq!(generator.osc2.transpose, Transpose::new(32));
        assert_eq!(generator.osc2.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.osc2.retrig_phase, RetrigPhase::default());
        assert_eq!(generator.osc2.volume, HexU50::parse("0x6B851E8E").unwrap());
        assert_eq!(generator.osc2.feedback, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.modulator1.transpose, Transpose::new(0));
        assert_eq!(generator.modulator1.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.modulator1.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.modulator1.amount, HexU50::parse("0xB333332A").unwrap());
        assert_eq!(generator.modulator1.feedback, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.modulator2.transpose, Transpose::new(-12));
        assert_eq!(generator.modulator2.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.modulator2.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.modulator2.amount, HexU50::parse("0xB851EB7B").unwrap());
        assert_eq!(generator.modulator2.feedback, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.modulator2_to_modulator1, OnOff::Off);
    }

    #[test]
    fn load_valid_sound_subtractive_sample() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT173.XML")).unwrap();
        let sound = load_synth_nodes(&xml_elements).unwrap();

        assert_eq!(&sound.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&sound.earliest_compatible_firmware.unwrap(), "3.1.0-beta");

        assert_eq!(sound.voice_priority, VoicePriority::High);
        assert_eq!(sound.polyphonic, Polyphony::Mono);
        assert_eq!(sound.volume, HexU50::parse("0x4CCCCCA8").unwrap());
        assert_eq!(sound.pan, Pan::parse("0x00000000").unwrap());
        assert_eq!(sound.portamento, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.modulation_fx, ModulationFx::Off);

        assert_eq!(sound.distorsion.saturation, ClippingAmount::new(0));
        assert_eq!(sound.distorsion.bit_crush, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.distorsion.decimation, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.unison.voice_count, UnisonVoiceCount::new(1));
        assert_eq!(sound.unison.detune, UnisonDetune::new(8));

        assert_eq!(sound.lfo1.rate, HexU50::parse("0x1999997E").unwrap());
        assert_eq!(sound.lfo1.shape, LfoShape::Triangle);
        assert_eq!(sound.lfo1.sync_level, SyncLevel::Off);
        assert_eq!(sound.lfo2.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.lfo2.shape, LfoShape::Triangle);

        assert_eq!(sound.envelope1.attack, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope1.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope1.sustain, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(sound.envelope1.release, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.envelope2.attack, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.sustain, HexU50::parse("0xFFFFFFE9").unwrap());
        assert_eq!(sound.envelope2.release, HexU50::parse("0xE6666654").unwrap());

        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Sixteenth);
        assert_eq!(sound.delay.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.sidechain.sync, SyncLevel::Sixteenth);
        assert_eq!(sound.sidechain.attack, AttackSidechain::try_from(327244).unwrap());
        assert_eq!(sound.sidechain.release, ReleaseSidechain::try_from(936).unwrap());
        assert_eq!(sound.sidechain.shape, HexU50::parse("0xDC28F5B2").unwrap());

        assert_eq!(sound.equalizer.bass_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.bass_frequency, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_frequency, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.arpeggiator.mode, ArpeggiatorMode::Off);
        assert_eq!(sound.arpeggiator.octaves_count, OctavesCount::new(2));
        assert_eq!(sound.arpeggiator.gate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.arpeggiator.rate, HexU50::parse("0x00000000").unwrap());

        let generator = sound.generator.as_subtractive().unwrap();

        assert_eq!(generator.lpf_mode, LpfMode::Lpf24);
        assert_eq!(generator.lpf_frequency, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(generator.lpf_resonance, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.hpf_frequency, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.hpf_resonance, HexU50::parse("0x80000000").unwrap());

        let sample = generator.osc1.as_sample().unwrap();

        assert_eq!(sample.transpose, Transpose::new(3));
        assert_eq!(sample.fine_transpose, FineTranspose::new(1));
        assert_eq!(sample.mode, SamplePlayMode::Cut);
        assert_eq!(sample.reversed, OnOff::Off);
        assert_eq!(sample.pitch_speed, PitchSpeed::Independent);
        assert_eq!(sample.time_stretch_amount, TimeStretchAmount::new(0));
        assert_eq!(sample.volume, HexU50::parse("0x7FFFFFFF").unwrap());

        let sample_one_zone = sample.sample.as_one_zone().unwrap();

        assert_eq!(
            sample_one_zone.file_path,
            String::from("SAMPLES/IDEAS/indicaymolena_bass_8882.wav")
        );
        assert_eq!(SamplePosition::new(1449984), sample_one_zone.zone.as_ref().unwrap().start);
        assert_eq!(SamplePosition::new(1511424), sample_one_zone.zone.as_ref().unwrap().end);

        let waveform = generator.osc2.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Square);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(0));
        assert_eq!(waveform.retrig_phase, RetrigPhase::default());
        assert_eq!(waveform.volume, HexU50::parse("0x80000000").unwrap());
    }

    #[test]
    fn load_valid_sound_subtractive_sample_sample_ranges() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT168A.XML")).unwrap();
        let sound = load_synth_nodes(&xml_elements).unwrap();

        assert_eq!(&sound.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&sound.earliest_compatible_firmware.unwrap(), "3.1.0-beta");

        assert_eq!(sound.voice_priority, VoicePriority::Medium);
        assert_eq!(sound.polyphonic, Polyphony::Poly);
        assert_eq!(sound.volume, HexU50::parse("0x4CCCCCA8").unwrap());
        assert_eq!(sound.pan, Pan::parse("0x00000000").unwrap());
        assert_eq!(sound.portamento, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.modulation_fx, ModulationFx::Off);

        assert_eq!(sound.distorsion.saturation, ClippingAmount::new(0));
        assert_eq!(sound.distorsion.bit_crush, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.distorsion.decimation, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.unison.voice_count, UnisonVoiceCount::new(1));
        assert_eq!(sound.unison.detune, UnisonDetune::new(8));

        assert_eq!(sound.lfo1.rate, HexU50::parse("0x1999997E").unwrap());
        assert_eq!(sound.lfo1.shape, LfoShape::Triangle);
        assert_eq!(sound.lfo1.sync_level, SyncLevel::Off);
        assert_eq!(sound.lfo2.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.lfo2.shape, LfoShape::Triangle);

        assert_eq!(sound.envelope1.attack, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope1.decay, HexU50::parse("0x7FFFFFD2").unwrap());
        assert_eq!(sound.envelope1.sustain, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope1.release, HexU50::parse("0x4C000000").unwrap());

        assert_eq!(sound.envelope2.attack, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.decay, HexU50::parse("0xE6666654").unwrap());
        assert_eq!(sound.envelope2.sustain, HexU50::parse("0xFFFFFFE9").unwrap());
        assert_eq!(sound.envelope2.release, HexU50::parse("0xE6666654").unwrap());

        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Sixteenth);
        assert_eq!(sound.delay.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.sidechain.sync, SyncLevel::Eighth);
        assert_eq!(sound.sidechain.attack, AttackSidechain::try_from(327244).unwrap());
        assert_eq!(sound.sidechain.release, ReleaseSidechain::try_from(936).unwrap());
        assert_eq!(sound.sidechain.shape, HexU50::parse("0xDC28F5B2").unwrap());

        assert_eq!(sound.equalizer.bass_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.bass_frequency, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_frequency, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.arpeggiator.mode, ArpeggiatorMode::Off);
        assert_eq!(sound.arpeggiator.octaves_count, OctavesCount::new(2));
        assert_eq!(sound.arpeggiator.gate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.arpeggiator.rate, HexU50::parse("0x00000000").unwrap());

        let generator = sound.generator.as_subtractive().unwrap();

        assert_eq!(generator.lpf_mode, LpfMode::Lpf24);
        assert_eq!(generator.lpf_frequency, HexU50::parse("0x50000000").unwrap());
        assert_eq!(generator.lpf_resonance, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.hpf_frequency, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.hpf_resonance, HexU50::parse("0x80000000").unwrap());

        let sample = generator.osc1.as_sample().unwrap();

        assert_eq!(sample.transpose, Transpose::default());
        assert_eq!(sample.fine_transpose, FineTranspose::default());
        assert_eq!(sample.mode, SamplePlayMode::Cut);
        assert_eq!(sample.reversed, OnOff::Off);
        assert_eq!(sample.pitch_speed, PitchSpeed::Independent);
        assert_eq!(sample.time_stretch_amount, TimeStretchAmount::new(0));
        assert_eq!(sample.volume, HexU50::parse("0x7FFFFFFF").unwrap());

        let sample_ranges = sample.sample.as_sample_ranges().unwrap();

        assert_eq!(2, sample_ranges.len());

        let sample_range = &sample_ranges[0];

        assert_eq!(sample_range.range_top_note.unwrap(), 72);
        assert_eq!(sample_range.file_path, "SAMPLES/Artists/Leonard Ludvigsen/Hangdrum/1.wav");
        assert_eq!(sample_range.zone.start, SamplePosition::new(0));
        assert_eq!(sample_range.zone.end, SamplePosition::new(146506));
        assert_eq!(sample_range.zone.start_loop.unwrap(), SamplePosition::new(19101));
        assert_eq!(sample_range.zone.end_loop.unwrap(), SamplePosition::new(19603));

        let sample_range = &sample_ranges[1];

        assert_eq!(sample_range.range_top_note, None);
        assert_eq!(sample_range.file_path, "SAMPLES/Artists/Leonard Ludvigsen/Hangdrum/2.wav");
        assert_eq!(sample_range.transpose, Transpose::new(-12));
        assert_eq!(sample_range.fine_transpose, FineTranspose::default());
        assert_eq!(sample_range.zone.start, SamplePosition::new(0));
        assert_eq!(sample_range.zone.end, SamplePosition::new(137227));
        assert_eq!(sample_range.zone.start_loop.unwrap(), SamplePosition::new(8089));
        assert_eq!(sample_range.zone.end_loop.unwrap(), SamplePosition::new(8256));

        let waveform = generator.osc2.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Square);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(0));
        assert_eq!(waveform.retrig_phase, RetrigPhase::default());
        assert_eq!(waveform.volume, HexU50::parse("0x80000000").unwrap());
    }
}
