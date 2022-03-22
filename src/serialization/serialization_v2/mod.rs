use crate::{
    values::{ArpeggiatorMode, HexU50, OctavesCount, OnOff, OscType, RetrigPhase, SoundType, SyncLevel},
    Arpeggiator, Chorus, Delay, Distorsion, Envelope, Equalizer, Error, Flanger, FmCarrier, FmGenerator, FmModulator, GateOutput,
    Kit, Lfo1, Lfo2, MidiOutput, ModKnob, ModulationFx, Oscillator, PatchCable, Phaser, RingModGenerator, Sample, SampleOneZone,
    SampleOscillator, SamplePosition, SampleRange, SampleZone, Sidechain, Sound, SoundGenerator, SoundSource,
    SubtractiveGenerator, Synth, Unison, WaveformOscillator, SoundOutput,
};
use xmltree::Element;

use super::{
    default_params::{DefaultParams, TwinSelector},
    keys,
    serialization_common::{convert_milliseconds_to_samples, parse_u8},
    xml,
};

/// Load a deluge synth XML file
pub fn load_synth_nodes(root_nodes: &[Element]) -> Result<Synth, Error> {
    let sound_node = xml::get_element(root_nodes, keys::SOUND)?;

    Ok(Synth {
        sound: load_sound(sound_node)?,
        firmware_version: xml::get_opt_element(root_nodes, keys::FIRMWARE_VERSION).map(xml::get_text),
        earliest_compatible_firmware: xml::get_opt_element(root_nodes, keys::EARLIEST_COMPATIBLE_FIRMWARE).map(xml::get_text),
    })
}

pub fn load_kit_nodes(roots: &[Element]) -> Result<Kit, Error> {
    let kit_node = xml::get_element(roots, keys::KIT)?;
    let sound_sources_node = xml::get_children_element(kit_node, keys::SOUND_SOURCES)?;
    let firmware_version = xml::get_opt_element(roots, keys::FIRMWARE_VERSION).map(xml::get_text);
    let earliest_compatible_firmware = xml::get_opt_element(roots, keys::EARLIEST_COMPATIBLE_FIRMWARE).map(xml::get_text);
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

fn load_sound(root: &Element) -> Result<Sound, Error> {
    let sound_type = xml::parse_children_element_content::<SoundType>(root, keys::MODE)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    let generator = match sound_type {
        SoundType::Subtractive => load_subtractive_sound(root)?,
        SoundType::Fm => load_fm_sound(root)?,
        SoundType::RingMod => load_ringmode_sound(root)?,
        _ => return Err(Error::UnsupportedSoundType),
    };

    Ok(Sound {
        polyphonic: xml::parse_children_element_content(root, keys::POLYPHONIC)?,
        voice_priority: xml::parse_children_element_content(root, keys::VOICE_PRIORITY)?,
        volume: xml::parse_children_element_content(default_params_node, keys::VOLUME)?,
        reverb_amount: xml::parse_children_element_content(default_params_node, keys::REVERB_AMOUNT)?,
        stutter_rate: xml::parse_children_element_content(default_params_node, keys::STUTTER_RATE)?,
        pan: xml::parse_children_element_content(default_params_node, keys::PAN)?,
        portamento: xml::parse_children_element_content(default_params_node, keys::PORTAMENTO)?,
        sidechain_send: xml::parse_opt_children_element_content(root, keys::SIDECHAIN_SEND)?,
        generator,
        envelope1: load_envelope(xml::get_children_element(default_params_node, keys::ENVELOPE1)?)?,
        envelope2: load_envelope(xml::get_children_element(default_params_node, keys::ENVELOPE2)?)?,
        lfo1: load_lfo1(xml::get_children_element(root, keys::LFO1)?, default_params_node)?,
        lfo2: load_lfo2(xml::get_children_element(root, keys::LFO2)?, default_params_node)?,
        unison: load_unison(xml::get_children_element(root, keys::UNISON)?)?,
        arpeggiator: load_arpeggiator(root, default_params_node)?,
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
        osc2_sync: xml::parse_opt_children_element_content(osc2_node, keys::OSCILLATOR_SYNC)?.unwrap_or(OnOff::Off),
        noise: xml::parse_children_element_content(default_params_node, keys::NOISE_VOLUME)?,
        lpf_mode: xml::parse_children_element_content(root, keys::LPF_MODE)?,
        lpf_frequency: xml::parse_children_element_content(default_params_node, keys::LPF_FREQUENCY)?,
        lpf_resonance: xml::parse_children_element_content(default_params_node, keys::LPF_RESONANCE)?,
        hpf_frequency: xml::parse_children_element_content(default_params_node, keys::HPF_FREQUENCY)?,
        hpf_resonance: xml::parse_children_element_content(default_params_node, keys::HPF_RESONANCE)?,
    }))
}

fn load_ringmode_sound(root: &Element) -> Result<SoundGenerator, Error> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    Ok(SoundGenerator::RingMod(RingModGenerator {
        osc1: load_oscillator(osc1_node, &DefaultParams::new(TwinSelector::A, default_params_node))?,
        osc2: load_oscillator(osc2_node, &DefaultParams::new(TwinSelector::B, default_params_node))?,
        osc2_sync: xml::parse_opt_children_element_content::<OnOff>(osc2_node, keys::OSCILLATOR_SYNC)?.unwrap_or(OnOff::Off),
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
        modulator2_to_modulator1: xml::parse_children_element_content(mod2_node, keys::FM_MOD1_TO_MOD2)?,
    }))
}

fn load_oscillator(root: &Element, params: &DefaultParams) -> Result<Oscillator, Error> {
    let osc_type = xml::parse_children_element_content(root, keys::TYPE)?;

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
        transpose: xml::parse_children_element_content(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_children_element_content(root, keys::CENTS)?,
        retrig_phase: xml::parse_opt_children_element_content(root, keys::RETRIG_PHASE)?.unwrap_or(RetrigPhase::Off),
        volume: params.parse_twin_children_content(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B)?,
        feedback: params.parse_twin_children_content(keys::FEEDBACK_CARRIER1, keys::FEEDBACK_CARRIER2)?,
    })
}

fn load_fm_modulation(root: &Element, params: &DefaultParams) -> Result<FmModulator, Error> {
    Ok(FmModulator {
        transpose: xml::parse_children_element_content(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_children_element_content(root, keys::CENTS)?,
        retrig_phase: xml::parse_children_element_content(root, keys::RETRIG_PHASE)?,
        amount: params.parse_twin_children_content(keys::AMOUNT_MODULATOR1, keys::AMOUNT_MODULATOR2)?,
        feedback: params.parse_twin_children_content(keys::FEEDBACK_MODULATOR1, keys::FEEDBACK_MODULATOR2)?,
    })
}

fn load_sample_oscillator(root: &Element, params: &DefaultParams) -> Result<Oscillator, Error> {
    Ok(Oscillator::Sample(SampleOscillator {
        transpose: xml::parse_opt_children_element_content(root, keys::TRANSPOSE)?.unwrap_or_default(),
        fine_transpose: xml::parse_opt_children_element_content(root, keys::CENTS)?.unwrap_or_default(),
        reversed: xml::parse_children_element_content(root, keys::REVERSED)?,
        mode: xml::parse_children_element_content(root, keys::LOOP_MODE)?,
        pitch_speed: xml::parse_children_element_content(root, keys::TIME_STRETCH_ENABLE)?,
        time_stretch_amount: xml::parse_children_element_content(root, keys::TIME_STRETCH_AMOUNT)?,
        sample: load_sample(root)?,
        linear_interpolation: xml::parse_opt_children_element_content(root, keys::LINEAR_INTERPOLATION)?.unwrap_or_default(),
        volume: params.parse_twin_children_content(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B)?,
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
                    range_top_note: xml::parse_opt_children_element_content(sample_range_node, keys::SAMPLE_RANGE_TOP_NOTE)?,
                    file_path: xml::parse_children_element_content(sample_range_node, keys::FILE_NAME)?,
                    transpose: xml::parse_opt_children_element_content(sample_range_node, keys::TRANSPOSE)?.unwrap_or_default(),
                    fine_transpose: xml::parse_opt_children_element_content(sample_range_node, keys::CENTS)?.unwrap_or_default(),
                    zone: parse_sample_zone(zone_node)?,
                };

                ranges.push(range);
            }

            Sample::SampleRanges(ranges)
        } else if let Some(sample_zone_node) = xml::get_opt_children_element(root, "zone") {
            Sample::OneZone(SampleOneZone {
                file_path: xml::parse_opt_children_element_content(root, keys::FILE_NAME)?.unwrap_or_default(),
                zone: Some(parse_sample_zone(sample_zone_node)?),
            })
        } else {
            Sample::OneZone(SampleOneZone {
                file_path: xml::parse_opt_children_element_content(root, keys::FILE_NAME)?.unwrap_or_default(),
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
    let start = SamplePosition::new(
        match xml::parse_opt_children_element_content::<u64>(root, keys::START_SAMPLES_POS)? {
            Some(samples) => samples,
            None => xml::parse_opt_children_element_content::<u64>(root, keys::START_MILLISECONDS_POS)?
                .map(convert_milliseconds_to_samples)
                .unwrap_or_default(),
        },
    );

    let end = SamplePosition::new(
        match xml::parse_opt_children_element_content::<u64>(root, keys::END_SAMPLES_POS)? {
            Some(samples) => samples,
            None => xml::parse_opt_children_element_content::<u64>(root, keys::END_MILLISECONDS_POS)?
                .map(convert_milliseconds_to_samples)
                .unwrap_or_default(),
        },
    );

    let start_loop = xml::parse_opt_children_element_content::<u64>(root, keys::START_LOOP_SAMPLES_POS)?.map(SamplePosition::new);
    let end_loop = xml::parse_opt_children_element_content::<u64>(root, keys::END_LOOP_SAMPLES_POS)?.map(SamplePosition::new);

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
        transpose: xml::parse_children_element_content(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_children_element_content(root, keys::CENTS)?,
        retrig_phase: xml::parse_opt_children_element_content(root, keys::RETRIG_PHASE)?.unwrap_or(RetrigPhase::Off),
        volume: params.parse_twin_children_content(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B)?,
        pulse_width: params.parse_twin_children_content(keys::PULSE_WIDTH_OSC_A, keys::PULSE_WIDTH_OSC_B)?,
    }))
}

fn load_midi_output(root: &Element) -> Result<MidiOutput, Error> {
    let channel = xml::parse_children_element_content(root, keys::CHANNEL).and_then(parse_u8)?;
    let note = xml::parse_children_element_content(root, keys::NOTE).and_then(parse_u8)?;

    Ok(MidiOutput { channel, note })
}

fn load_gate_output(root: &Element) -> Result<GateOutput, Error> {
    xml::get_children_element_content(root, keys::CHANNEL)
        .and_then(|s| parse_u8(&s))
        .map(|channel| GateOutput { channel })
}

fn load_sound_output(root: &Element) -> Result<SoundOutput, Error> {
    Ok(SoundOutput {
        sound: Box::new(load_sound(root)?),
        name: xml::parse_children_element_content(root, keys::NAME)?,
    })
}

fn load_sound_source(root: &Element) -> Result<SoundSource, Error> {
    Ok(match root.name.as_str() {
        keys::SOUND => SoundSource::SoundOutput(load_sound_output(root)?),
        keys::MIDI_OUTPUT => SoundSource::MidiOutput(load_midi_output(root)?),
        keys::GATE_OUTPUT => SoundSource::GateOutput(load_gate_output(root)?),
        _ => return Err(Error::UnsupportedSoundSource(root.name.clone())),
    })
}

fn load_envelope(root: &Element) -> Result<Envelope, Error> {
    Ok(Envelope {
        attack: xml::parse_children_element_content(root, keys::ENV_ATTACK)?,
        decay: xml::parse_children_element_content(root, keys::ENV_DECAY)?,
        sustain: xml::parse_children_element_content(root, keys::ENV_SUSTAIN)?,
        release: xml::parse_children_element_content(root, keys::ENV_RELEASE)?,
    })
}

fn load_lfo1(root: &Element, default_params_node: &Element) -> Result<Lfo1, Error> {
    Ok(Lfo1 {
        shape: xml::parse_children_element_content(root, keys::LFO_SHAPE)?,
        sync_level: xml::parse_children_element_content(root, keys::SYNC_LEVEL)?,
        rate: xml::parse_children_element_content(default_params_node, keys::LFO1_RATE)?,
    })
}

fn load_lfo2(root: &Element, default_params_node: &Element) -> Result<Lfo2, Error> {
    Ok(Lfo2 {
        shape: xml::parse_children_element_content(root, keys::LFO_SHAPE)?,
        rate: xml::parse_children_element_content(default_params_node, keys::LFO2_RATE)?,
    })
}

fn load_unison(root: &Element) -> Result<Unison, Error> {
    Ok(Unison {
        voice_count: xml::parse_children_element_content(root, keys::UNISON_VOICE_COUNT)?,
        detune: xml::parse_children_element_content(root, keys::UNISON_DETUNE)?,
    })
}

fn load_delay(root: &Element, default_params_node: &Element) -> Result<Delay, Error> {
    Ok(Delay {
        ping_pong: xml::parse_children_element_content(root, keys::PING_PONG)?,
        analog: xml::parse_children_element_content(root, keys::ANALOG)?,
        sync_level: xml::parse_children_element_content(root, keys::SYNC_LEVEL)?,
        amount: xml::parse_children_element_content(default_params_node, keys::DELAY_FEEDBACK)?,
        rate: xml::parse_children_element_content(default_params_node, keys::DELAY_RATE)?,
    })
}

fn load_arpeggiator(root: &Element, default_params_node: &Element) -> Result<Arpeggiator, Error> {
    Ok(match xml::get_opt_children_element(root, keys::ARPEGGIATOR) {
        Some(arpeggiator_node) => Arpeggiator {
            mode: xml::parse_children_element_content(arpeggiator_node, keys::ARPEGGIATOR_MODE)?,
            sync_level: xml::parse_children_element_content(arpeggiator_node, keys::SYNC_LEVEL)?,
            octaves_count: xml::parse_children_element_content(arpeggiator_node, keys::ARPEGGIATOR_OCTAVE_COUNT)?,
            rate: xml::parse_children_element_content(default_params_node, keys::ARPEGGIATOR_RATE)?,
            gate: xml::parse_children_element_content(default_params_node, keys::ARPEGGIATOR_GATE)?,
        },
        None => Arpeggiator {
            mode: ArpeggiatorMode::Off,
            sync_level: SyncLevel::Sixteenth,
            octaves_count: OctavesCount::default(),
            rate: HexU50::new(25),
            gate: HexU50::new(25),
        },
    })
}

fn load_distorsion(root: &Element, default_params_node: &Element) -> Result<Distorsion, Error> {
    Ok(Distorsion {
        saturation: xml::parse_opt_children_element_content(root, keys::CLIPPING_AMOUNT)?.unwrap_or_default(),
        bit_crush: xml::parse_children_element_content(default_params_node, keys::BIT_CRUSH)?,
        decimation: xml::parse_children_element_content(default_params_node, keys::DECIMATION)?,
    })
}

fn load_equalizer(root: &Element) -> Result<Equalizer, Error> {
    Ok(Equalizer {
        bass_level: xml::parse_children_element_content(root, keys::EQ_BASS)?,
        bass_frequency: xml::parse_children_element_content(root, keys::EQ_BASS_FREQUENCY)?,
        treble_level: xml::parse_children_element_content(root, keys::EQ_TREBLE)?,
        treble_frequency: xml::parse_children_element_content(root, keys::EQ_TREBLE_FREQUENCY)?,
    })
}

fn load_modulation_fx(root: &Element) -> Result<ModulationFx, Error> {
    let modulation_fx_type = xml::parse_children_element_content(root, keys::MOD_FX_TYPE)?;
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
        rate: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_RATE)?,
        feedback: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_FEEDBACK)?,
    })
}

fn load_modulation_fx_chorus(default_params_node: &Element) -> Result<Chorus, Error> {
    Ok(Chorus {
        rate: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_RATE)?,
        depth: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_DEPTH)?,
        offset: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_OFFSET)?,
    })
}

fn load_modulation_fx_phaser(default_params_node: &Element) -> Result<Phaser, Error> {
    Ok(Phaser {
        rate: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_RATE)?,
        depth: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_DEPTH)?,
        feedback: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_FEEDBACK)?,
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
        control_param: xml::parse_children_element_content(element, keys::MOD_KNOB_CONTROL_PARAM)?,
        patch_amount_from_source: xml::parse_opt_children_element_content(element, keys::MOD_KNOB_PATCH_AMOUNT_FROM_SOURCE)?,
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
        source: xml::parse_children_element_content(root, keys::PATCH_CABLE_SOURCE)?,
        destination: xml::parse_children_element_content(root, keys::PATCH_CABLE_DESTINATION)?,
        amount: xml::parse_children_element_content(root, keys::PATCH_CABLE_AMOUNT)?,
    })
}

fn load_sidechain(root: &Element, default_params_node: &Element) -> Result<Sidechain, Error> {
    Ok(Sidechain {
        attack: xml::parse_children_element_content(root, keys::COMPRESSOR_ATTACK)?,
        release: xml::parse_children_element_content(root, keys::COMPRESSOR_RELEASE)?,
        shape: xml::parse_children_element_content(default_params_node, keys::COMPRESSOR_SHAPE)?,
        sync: xml::parse_children_element_content(root, keys::COMPRESSOR_SYNCLEVEL)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        load_synth, save_synth,
        values::{
            AttackSidechain, ClippingAmount, FineTranspose, LfoShape, LpfMode, Pan, Polyphony, ReleaseSidechain, RetrigPhase,
            Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
        },
    };

    use super::*;

    #[test]
    fn load_valid_kit_xml() {
        let roots = xml::load_xml(include_str!("../../data_tests/KITS/KIT026.XML")).unwrap();

        assert!(load_kit_nodes(&roots).is_ok());
    }

    #[test]
    fn load_save_load_sound_subtractive() {
        let synth = load_synth(include_str!("../../data_tests/SYNTHS/SYNT061.XML")).unwrap();
        let xml = save_synth(&synth).unwrap();
        let reloaded_synth = load_synth(&xml).unwrap();

        assert_eq!(reloaded_synth, synth);
    }

    #[test]
    fn load_valid_sound_subtractive() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT061.XML")).unwrap();
        let synth = load_synth_nodes(&xml_elements).unwrap();
        let sound = &synth.sound;

        assert_eq!(&synth.firmware_version.unwrap(), "2.0.0-beta");
        assert_eq!(&synth.earliest_compatible_firmware.unwrap(), "2.0.0-beta");

        assert_eq!(sound.voice_priority, VoicePriority::Medium);
        assert_eq!(sound.polyphonic, Polyphony::Poly);
        assert_eq!(sound.volume, HexU50::parse("0x6C000000").unwrap());
        assert_eq!(sound.pan, Pan::parse("0x00000000").unwrap());
        assert_eq!(sound.portamento, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.modulation_fx, ModulationFx::Off);

        assert_eq!(sound.distorsion.saturation, ClippingAmount::new(5));
        assert_eq!(sound.distorsion.bit_crush, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.distorsion.decimation, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.lfo1.rate, HexU50::parse("0x20000000").unwrap());
        assert_eq!(sound.lfo1.shape, LfoShape::Triangle);
        assert_eq!(sound.lfo1.sync_level, SyncLevel::Off);
        assert_eq!(sound.lfo2.rate, HexU50::parse("0x33333313").unwrap());
        assert_eq!(sound.lfo2.shape, LfoShape::Triangle);

        assert_eq!(sound.envelope1.attack, HexU50::parse("0x12000000").unwrap());
        assert_eq!(sound.envelope1.decay, HexU50::parse("0xF5C28F47").unwrap());
        assert_eq!(sound.envelope1.sustain, HexU50::parse("0x51EB84F9").unwrap());
        assert_eq!(sound.envelope1.release, HexU50::parse("0xF8000000").unwrap());

        assert_eq!(sound.envelope2.attack, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.envelope2.decay, HexU50::parse("0x8C000000").unwrap());
        assert_eq!(sound.envelope2.sustain, HexU50::parse("0xC7AE146E").unwrap());
        assert_eq!(sound.envelope2.release, HexU50::parse("0x51EB84F9").unwrap());

        assert_eq!(sound.unison.voice_count, UnisonVoiceCount::new(3));
        assert_eq!(sound.unison.detune, UnisonDetune::new(13));

        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Eighth);
        assert_eq!(sound.delay.rate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());

        assert_eq!(sound.sidechain.sync, SyncLevel::Eighth);
        assert_eq!(sound.sidechain.attack, AttackSidechain::try_from(327244).unwrap());
        assert_eq!(sound.sidechain.release, ReleaseSidechain::try_from(936).unwrap());

        assert_eq!(sound.equalizer.bass_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.bass_frequency, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_frequency, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.arpeggiator.mode, ArpeggiatorMode::Off);
        assert_eq!(sound.arpeggiator.octaves_count, OctavesCount::new(1));
        assert_eq!(sound.arpeggiator.gate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.arpeggiator.rate, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.delay.rate, HexU50::parse("0xFFFFFFE9").unwrap());
        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Eighth);

        let generator = sound.generator.as_subtractive().unwrap();

        assert_eq!(generator.lpf_mode, LpfMode::Lpf24);
        assert_eq!(generator.lpf_frequency, HexU50::parse("0x24000000").unwrap());
        assert_eq!(generator.lpf_resonance, HexU50::parse("0x82000000").unwrap());
        assert_eq!(generator.hpf_frequency, HexU50::parse("0x1C000000").unwrap());
        assert_eq!(generator.hpf_resonance, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.osc2_sync, OnOff::Off);

        let waveform = generator.osc1.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Square);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(0));
        assert_eq!(waveform.retrig_phase, RetrigPhase::new(0));
        assert_eq!(waveform.volume, HexU50::parse("0x70A3D6DF").unwrap());
        assert_eq!(waveform.pulse_width, HexU50::parse("0x00000000").unwrap());

        let waveform = generator.osc2.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Saw);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(8));
        assert_eq!(waveform.retrig_phase, RetrigPhase::new(0));
        assert_eq!(waveform.volume, HexU50::parse("0x7FFFFFD2").unwrap());
        assert_eq!(waveform.pulse_width, HexU50::parse("0x00000000").unwrap());

        assert_eq!(3, sound.cables.len());

        assert_eq!("velocity", sound.cables[0].source);
        assert_eq!("volume", sound.cables[0].destination);
        assert_eq!(HexU50::parse("0x3FFFFFE8").unwrap(), sound.cables[0].amount);

        assert_eq!("lfo1", sound.cables[1].source);
        assert_eq!("pitch", sound.cables[1].destination);
        assert_eq!(HexU50::parse("0x03000000").unwrap(), sound.cables[1].amount);

        assert_eq!("envelope2", sound.cables[2].source);
        assert_eq!("lpfFrequency", sound.cables[2].destination);
        assert_eq!(HexU50::parse("0x251EB844").unwrap(), sound.cables[2].amount);
    }

    #[test]
    fn load_valid_sound_fm() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT167.XML")).unwrap();
        let synth = load_synth_nodes(&xml_elements).unwrap();
        let sound = &synth.sound;
        let generator = sound.generator.as_fm().unwrap();

        assert_eq!(generator.osc1.transpose, Transpose::new(0));
        assert_eq!(generator.osc1.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.osc1.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.osc1.feedback, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.osc1.volume, HexU50::parse("0x7FFFFFFF").unwrap());

        assert_eq!(generator.osc2.transpose, Transpose::new(0));
        assert_eq!(generator.osc2.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.osc2.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.osc2.feedback, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.osc2.volume, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.modulator1.transpose, Transpose::new(-15));
        assert_eq!(generator.modulator1.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.modulator1.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.modulator2_to_modulator1, OnOff::Off);

        assert_eq!(generator.modulator2.transpose, Transpose::new(-12));
        assert_eq!(generator.modulator2.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.modulator2.retrig_phase, RetrigPhase::new(0));
    }
}
