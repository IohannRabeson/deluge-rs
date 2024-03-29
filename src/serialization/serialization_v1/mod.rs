use crate::{
    values::{
        ArpeggiatorMode, AttackSidechain, HexU50, MidiChannel, ModulationFxType, OnOff, OscType, Pan, ReleaseSidechain,
        RetrigPhase, SamplePosition, SyncLevel, SynthMode,
    },
    Arpeggiator, Chorus, CvGateRow, Delay, Distorsion, Envelope, Equalizer, Flanger, FmCarrier, FmModulator, FmSynth, Hpf, Kit,
    Lfo1, Lfo2, Lpf, MidiRow, ModKnob, ModulationFx, PatchCable, Phaser, RingModSynth, RowKit, Sample, SampleOneZone,
    SampleOscillator, SampleRange, SampleZone, SerializationError, Sidechain, Sound, SoundRow, SubtractiveOscillator,
    SubtractiveSynth, Synth, SynthEngine, Unison, WaveformOscillator,
};
use xmltree::Element;

use super::{
    default_params::{DefaultParams, TwinSelector},
    keys,
    serialization_common::convert_milliseconds_to_samples,
    xml,
};

/// Load a deluge synth XML file
pub fn load_synth_nodes(root_nodes: &[Element]) -> Result<Synth, SerializationError> {
    let sound_node = xml::get_element(root_nodes, keys::SOUND)?;

    Ok(Synth {
        sound: load_sound(sound_node)?,
    })
}

pub fn load_kit_nodes(roots: &[Element]) -> Result<Kit, SerializationError> {
    let kit_node = xml::get_element(roots, keys::KIT)?;
    let sound_sources_node = xml::get_children_element(kit_node, keys::SOUND_SOURCES)?;
    let sources: Vec<Result<RowKit, SerializationError>> = sound_sources_node
        .children
        .iter()
        .filter_map(xml::keep_element_only)
        .map(load_sound_source)
        .collect();

    if let Some(result_with_error) = sources.iter().find(|s| s.is_err()) {
        return Err(result_with_error
            .as_ref()
            .unwrap_err()
            .clone());
    }

    return Ok(Kit {
        rows: sources
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<RowKit>>(),
        lpf_mode: xml::parse_children_element_content(kit_node, keys::LPF_MODE)?,
        modulation_fx: load_modulation_fx(kit_node)?,
        current_filter_type: xml::parse_children_element_content(kit_node, keys::CURRENT_FILTER_TYPE)?,
        selected_row_index: xml::parse_children_element_content(kit_node, keys::SELECTED_DRUM_INDEX)?,
        volume: load_global_hexu(kit_node, keys::VOLUME)?,
        reverb_amount: load_global_hexu(kit_node, keys::REVERB_AMOUNT)?,
        pan: load_global_pan(kit_node)?,
        bit_crush: load_global_hexu(kit_node, keys::BIT_CRUSH)?,
        decimation: load_global_hexu(kit_node, keys::DECIMATION)?,
        stutter_rate: load_global_hexu(kit_node, keys::STUTTER_RATE)?,
        delay: load_global_delay(kit_node)?,
        sidechain: Sidechain::default(),
        lpf: load_global_lpf(kit_node)?,
        hpf: load_global_hpf(kit_node)?,
        equalizer: load_global_equalizer(kit_node)?,
    });
}

fn load_sound(root: &Element) -> Result<Sound, SerializationError> {
    let sound_type = xml::parse_opt_children_element_content::<SynthMode>(root, keys::MODE)?.unwrap_or(SynthMode::Subtractive);
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    let generator = match sound_type {
        SynthMode::Subtractive => load_subtractive_sound(root)?,
        SynthMode::Fm => load_fm_sound(root)?,
        SynthMode::RingMod => load_ringmode_sound(root)?,
        _ => return Err(SerializationError::UnsupportedSoundType),
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
        arpeggiator: create_default_arpeggiator(),
        delay: load_delay(xml::get_children_element(root, keys::DELAY)?, default_params_node)?,
        distorsion: load_distorsion(root, default_params_node)?,
        equalizer: load_equalizer(xml::get_children_element(default_params_node, keys::EQUALIZER)?)?,
        modulation_fx: load_modulation_fx(root)?,
        sidechain: create_default_sidechain(),
        cables: load_patch_cables(xml::get_children_element(default_params_node, keys::PATCH_CABLES)?)?,
        mod_knobs: load_mod_knobs(xml::get_children_element(root, keys::MOD_KNOBS)?)?,
    })
}

fn create_default_arpeggiator() -> Arpeggiator {
    Arpeggiator {
        mode: ArpeggiatorMode::Off,
        sync_level: SyncLevel::Sixteenth,
        octaves_count: 2.into(),
        rate: 25.into(),
        gate: 25.into(),
    }
}

fn create_default_sidechain() -> Sidechain {
    Sidechain {
        attack: AttackSidechain::try_from(327244).unwrap(),
        release: ReleaseSidechain::try_from(936).unwrap(),
        shape: HexU50::parse("0xDC28F5B2").unwrap(),
        sync: SyncLevel::Sixteenth,
    }
}

fn load_subtractive_sound(root: &Element) -> Result<SynthEngine, SerializationError> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;
    let mut osc1 = load_oscillator(osc1_node, &DefaultParams::new(TwinSelector::A, default_params_node))?;
    let mut osc2 = load_oscillator(osc2_node, &DefaultParams::new(TwinSelector::B, default_params_node))?;

    load_oscillator_reset_osc(root, &mut osc1, &mut osc2)?;

    Ok(SynthEngine::from(SubtractiveSynth {
        osc1,
        osc2,
        osc2_sync: xml::parse_opt_children_element_content(osc2_node, keys::OSCILLATOR_SYNC)?.unwrap_or(OnOff::Off),
        noise: xml::parse_children_element_content(default_params_node, keys::NOISE_VOLUME)?,
        lpf_mode: xml::parse_children_element_content(root, keys::LPF_MODE)?,
        lpf_frequency: xml::parse_children_element_content(default_params_node, keys::LPF_FREQUENCY)?,
        lpf_resonance: xml::parse_children_element_content(default_params_node, keys::LPF_RESONANCE)?,
        hpf_frequency: xml::parse_children_element_content(default_params_node, keys::HPF_FREQUENCY)?,
        hpf_resonance: xml::parse_children_element_content(default_params_node, keys::HPF_RESONANCE)?,
        osc1_volume: xml::parse_children_element_content(default_params_node, keys::VOLUME_OSC_A)?,
        osc2_volume: xml::parse_children_element_content(default_params_node, keys::VOLUME_OSC_B)?,
    }))
}

fn assign_retrig_phase(mut osc: &mut SubtractiveOscillator, retrig_phase: RetrigPhase) {
    if let SubtractiveOscillator::Waveform(osc) = &mut osc {
        osc.retrig_phase = retrig_phase;
    }
}

pub(crate) fn load_ringmode_sound(root: &Element) -> Result<SynthEngine, SerializationError> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let osc1_type: OscType = xml::parse_children_element_content(osc1_node, keys::TYPE)?;
    let osc2_type: OscType = xml::parse_children_element_content(osc2_node, keys::TYPE)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;
    let mut osc1 = load_waveform_oscillator_imp(
        osc1_type,
        osc1_node,
        &DefaultParams::new(TwinSelector::A, default_params_node),
    )?;
    let mut osc2 = load_waveform_oscillator_imp(
        osc2_type,
        osc2_node,
        &DefaultParams::new(TwinSelector::B, default_params_node),
    )?;

    load_oscillator_reset_waveform_osc(root, &mut osc1, &mut osc2)?;

    Ok(SynthEngine::from(RingModSynth {
        osc1,
        osc2,
        osc2_sync: xml::parse_opt_children_element_content::<OnOff>(osc2_node, keys::OSCILLATOR_SYNC)?.unwrap_or(OnOff::Off),
        noise: xml::parse_children_element_content(default_params_node, keys::NOISE_VOLUME)?,
    }))
}

fn load_oscillator_reset_osc(
    root: &Element,
    osc1: &mut SubtractiveOscillator,
    osc2: &mut SubtractiveOscillator,
) -> Result<(), SerializationError> {
    if let Some(oscillator_reset_node) = xml::parse_opt_children_element_content::<OnOff>(root, keys::OSCILLATOR_RESET)? {
        let retrig_phase = retrig_phase_from_oscillator_reset(oscillator_reset_node);

        assign_retrig_phase(osc1, retrig_phase);
        assign_retrig_phase(osc2, retrig_phase);
    }

    Ok(())
}

fn load_oscillator_reset_waveform_osc(
    root: &Element,
    osc1: &mut WaveformOscillator,
    osc2: &mut WaveformOscillator,
) -> Result<(), SerializationError> {
    if let Some(oscillator_reset_node) = xml::parse_opt_children_element_content::<OnOff>(root, keys::OSCILLATOR_RESET)? {
        let retrig_phase = retrig_phase_from_oscillator_reset(oscillator_reset_node);

        osc1.retrig_phase = retrig_phase;
        osc2.retrig_phase = retrig_phase;
    }

    Ok(())
}

pub(crate) fn load_fm_sound(root: &Element) -> Result<SynthEngine, SerializationError> {
    let osc1_node = xml::get_children_element(root, keys::OSC1)?;
    let osc2_node = xml::get_children_element(root, keys::OSC2)?;
    let mod1_node = xml::get_children_element(root, keys::FM_MODULATOR1)?;
    let mod2_node = xml::get_children_element(root, keys::FM_MODULATOR2)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;
    let params_a = &DefaultParams::new(TwinSelector::A, default_params_node);
    let params_b = &DefaultParams::new(TwinSelector::B, default_params_node);
    let mut osc1 = load_carrier(osc1_node, params_a)?;
    let mut osc2 = load_carrier(osc2_node, params_b)?;

    load_oscillator_reset_carrier(root, &mut osc1, &mut osc2)?;

    Ok(SynthEngine::from(FmSynth {
        osc1: load_carrier(osc1_node, params_a)?,
        osc2: load_carrier(osc2_node, params_b)?,
        modulator1: load_fm_modulation(mod1_node, params_a)?,
        modulator2: load_fm_modulation(mod2_node, params_b)?,
        modulator2_to_modulator1: xml::parse_children_element_content(mod2_node, keys::FM_MOD1_TO_MOD2)?,
        osc1_volume: xml::parse_children_element_content(default_params_node, keys::VOLUME_OSC_A)?,
        osc2_volume: xml::parse_children_element_content(default_params_node, keys::VOLUME_OSC_B)?,
    }))
}

fn load_oscillator_reset_carrier(
    root: &Element,
    mut osc1: &mut FmCarrier,
    mut osc2: &mut FmCarrier,
) -> Result<(), SerializationError> {
    if let Some(oscillator_reset_node) = xml::parse_opt_children_element_content::<OnOff>(root, keys::OSCILLATOR_RESET)? {
        let retrig_phase = retrig_phase_from_oscillator_reset(oscillator_reset_node);

        osc1.retrig_phase = retrig_phase;
        osc2.retrig_phase = retrig_phase;
    }

    Ok(())
}

fn retrig_phase_from_oscillator_reset(oscillator_reset_node: OnOff) -> RetrigPhase {
    match oscillator_reset_node {
        OnOff::On => RetrigPhase::Degrees(0),
        OnOff::Off => RetrigPhase::Off,
    }
}

pub(crate) fn load_oscillator(root: &Element, params: &DefaultParams) -> Result<SubtractiveOscillator, SerializationError> {
    let osc_type: OscType = xml::parse_children_element_content(root, keys::TYPE)?;

    match osc_type {
        OscType::Sample => load_sample_oscillator(root),
        OscType::AnalogSaw => load_waveform_oscillator(osc_type, root, params),
        OscType::AnalogSquare => load_waveform_oscillator(osc_type, root, params),
        OscType::Saw => load_waveform_oscillator(osc_type, root, params),
        OscType::Sine => load_waveform_oscillator(osc_type, root, params),
        OscType::Square => load_waveform_oscillator(osc_type, root, params),
        OscType::Triangle => load_waveform_oscillator(osc_type, root, params),
    }
}

fn load_carrier(root: &Element, params: &DefaultParams) -> Result<FmCarrier, SerializationError> {
    Ok(FmCarrier {
        transpose: xml::parse_children_element_content(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_children_element_content(root, keys::CENTS)?,
        retrig_phase: xml::parse_opt_children_element_content(root, keys::RETRIG_PHASE)?.unwrap_or(RetrigPhase::Off),
        feedback: params.parse_twin_children_content(keys::FEEDBACK_CARRIER1, keys::FEEDBACK_CARRIER2)?,
    })
}

fn load_fm_modulation(root: &Element, params: &DefaultParams) -> Result<FmModulator, SerializationError> {
    Ok(FmModulator {
        transpose: xml::parse_children_element_content(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_children_element_content(root, keys::CENTS)?,
        retrig_phase: xml::parse_opt_children_element_content(root, keys::RETRIG_PHASE)?.unwrap_or_default(),
        amount: params.parse_twin_children_content(keys::AMOUNT_MODULATOR1, keys::AMOUNT_MODULATOR2)?,
        feedback: params.parse_twin_children_content(keys::FEEDBACK_MODULATOR1, keys::FEEDBACK_MODULATOR2)?,
    })
}

fn load_sample_oscillator(root: &Element) -> Result<SubtractiveOscillator, SerializationError> {
    Ok(SubtractiveOscillator::Sample(SampleOscillator {
        transpose: xml::parse_opt_children_element_content(root, keys::TRANSPOSE)?.unwrap_or_default(),
        fine_transpose: xml::parse_opt_children_element_content(root, keys::CENTS)?.unwrap_or_default(),
        reversed: xml::parse_children_element_content(root, keys::REVERSED)?,
        mode: xml::parse_children_element_content(root, keys::LOOP_MODE)?,
        pitch_speed: xml::parse_children_element_content(root, keys::TIME_STRETCH_ENABLE)?,
        time_stretch_amount: xml::parse_children_element_content(root, keys::TIME_STRETCH_AMOUNT)?,
        sample: load_sample(root)?,
        linear_interpolation: xml::parse_opt_children_element_content(root, keys::LINEAR_INTERPOLATION)?.unwrap_or_default(),
    }))
}

fn load_sample(root: &Element) -> Result<Sample, SerializationError> {
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
fn parse_sample_zone(root: &Element) -> Result<SampleZone, SerializationError> {
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

fn load_waveform_oscillator(
    osc_type: OscType,
    root: &Element,
    params: &DefaultParams,
) -> Result<SubtractiveOscillator, SerializationError> {
    Ok(SubtractiveOscillator::Waveform(load_waveform_oscillator_imp(
        osc_type, root, params,
    )?))
}

fn load_waveform_oscillator_imp(
    osc_type: OscType,
    root: &Element,
    params: &DefaultParams,
) -> Result<WaveformOscillator, SerializationError> {
    Ok(WaveformOscillator {
        osc_type,
        transpose: xml::parse_children_element_content(root, keys::TRANSPOSE)?,
        fine_transpose: xml::parse_children_element_content(root, keys::CENTS)?,
        retrig_phase: xml::parse_opt_children_element_content(root, keys::RETRIG_PHASE)?.unwrap_or(RetrigPhase::Off),
        pulse_width: params.parse_twin_children_content(keys::PULSE_WIDTH_OSC_A, keys::PULSE_WIDTH_OSC_B)?,
    })
}

fn load_midi_output(root: &Element) -> Result<MidiRow, SerializationError> {
    let channel: MidiChannel = xml::parse_children_element_content(root, keys::CHANNEL)?;
    let note = xml::parse_children_element_content(root, keys::NOTE)?;

    Ok(MidiRow { channel, note })
}

fn load_gate_output(root: &Element) -> Result<CvGateRow, SerializationError> {
    Ok(CvGateRow::new(xml::parse_children_element_content(root, keys::CHANNEL)?))
}

fn load_sound_output(root: &Element) -> Result<SoundRow, SerializationError> {
    Ok(SoundRow {
        sound: Box::new(load_sound(root)?),
        name: xml::parse_children_element_content(root, keys::NAME)?,
    })
}

pub(crate) fn load_sound_source(root: &Element) -> Result<RowKit, SerializationError> {
    Ok(match root.name.as_str() {
        keys::SOUND => RowKit::Sound(load_sound_output(root)?),
        keys::MIDI_OUTPUT => RowKit::Midi(load_midi_output(root)?),
        keys::GATE_OUTPUT => RowKit::CvGate(load_gate_output(root)?),
        _ => return Err(SerializationError::UnsupportedSoundSource(root.name.clone())),
    })
}

pub(crate) fn load_envelope(root: &Element) -> Result<Envelope, SerializationError> {
    Ok(Envelope {
        attack: xml::parse_children_element_content(root, keys::ENV_ATTACK)?,
        decay: xml::parse_children_element_content(root, keys::ENV_DECAY)?,
        sustain: xml::parse_children_element_content(root, keys::ENV_SUSTAIN)?,
        release: xml::parse_children_element_content(root, keys::ENV_RELEASE)?,
    })
}

pub(crate) fn load_lfo1(root: &Element, default_params_node: &Element) -> Result<Lfo1, SerializationError> {
    Ok(Lfo1 {
        shape: xml::parse_children_element_content(root, keys::LFO_SHAPE)?,
        sync_level: xml::parse_children_element_content(root, keys::SYNC_LEVEL)?,
        rate: xml::parse_children_element_content(default_params_node, keys::LFO1_RATE)?,
    })
}

pub(crate) fn load_lfo2(root: &Element, default_params_node: &Element) -> Result<Lfo2, SerializationError> {
    Ok(Lfo2 {
        shape: xml::parse_children_element_content(root, keys::LFO_SHAPE)?,
        rate: xml::parse_children_element_content(default_params_node, keys::LFO2_RATE)?,
    })
}

pub(crate) fn load_unison(root: &Element) -> Result<Unison, SerializationError> {
    Ok(Unison {
        voice_count: xml::parse_children_element_content(root, keys::UNISON_VOICE_COUNT)?,
        detune: xml::parse_children_element_content(root, keys::UNISON_DETUNE)?,
    })
}

pub(crate) fn load_delay(root: &Element, default_params_node: &Element) -> Result<Delay, SerializationError> {
    Ok(Delay {
        ping_pong: xml::parse_children_element_content(root, keys::PING_PONG)?,
        analog: xml::parse_children_element_content(root, keys::ANALOG)?,
        sync_level: xml::parse_children_element_content(root, keys::SYNC_LEVEL)?,
        amount: xml::parse_children_element_content(default_params_node, keys::DELAY_FEEDBACK)?,
        rate: xml::parse_children_element_content(default_params_node, keys::DELAY_RATE)?,
    })
}

fn load_global_delay(kit_node: &Element) -> Result<Delay, SerializationError> {
    let default_params_node = xml::get_children_element(kit_node, keys::DEFAULT_PARAMS)?;
    let default_delay_node = xml::get_children_element(default_params_node, keys::DELAY)?;

    Ok(Delay {
        ping_pong: OnOff::On,
        analog: OnOff::Off,
        sync_level: SyncLevel::Sixteenth,
        amount: xml::parse_children_element_content(default_delay_node, keys::FEEDBACK)?,
        rate: xml::parse_children_element_content(default_delay_node, keys::RATE)?,
    })
}

pub(crate) fn load_distorsion(root: &Element, default_params_node: &Element) -> Result<Distorsion, SerializationError> {
    Ok(Distorsion {
        saturation: xml::parse_opt_children_element_content(root, keys::CLIPPING_AMOUNT)?.unwrap_or_default(),
        bit_crush: xml::parse_children_element_content(default_params_node, keys::BIT_CRUSH)?,
        decimation: xml::parse_children_element_content(default_params_node, keys::DECIMATION)?,
    })
}

pub(crate) fn load_equalizer(root: &Element) -> Result<Equalizer, SerializationError> {
    Ok(Equalizer {
        bass_level: xml::parse_children_element_content(root, keys::EQ_BASS)?,
        bass_frequency: xml::parse_children_element_content(root, keys::EQ_BASS_FREQUENCY)?,
        treble_level: xml::parse_children_element_content(root, keys::EQ_TREBLE)?,
        treble_frequency: xml::parse_children_element_content(root, keys::EQ_TREBLE_FREQUENCY)?,
    })
}

pub(crate) fn load_modulation_fx(root: &Element) -> Result<ModulationFx, SerializationError> {
    let modulation_fx_type: ModulationFxType = xml::parse_children_element_content(root, keys::MOD_FX_TYPE)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    Ok(match modulation_fx_type {
        ModulationFxType::Off => ModulationFx::Off,
        ModulationFxType::Flanger => ModulationFx::Flanger(load_modulation_fx_flanger(default_params_node)?),
        ModulationFxType::Chorus => ModulationFx::Chorus(load_modulation_fx_chorus(default_params_node)?),
        ModulationFxType::Phaser => ModulationFx::Phaser(load_modulation_fx_phaser(default_params_node)?),
    })
}

fn load_modulation_fx_flanger(default_params_node: &Element) -> Result<Flanger, SerializationError> {
    Ok(Flanger {
        rate: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_RATE)?,
        feedback: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_FEEDBACK)?,
    })
}

fn load_modulation_fx_chorus(default_params_node: &Element) -> Result<Chorus, SerializationError> {
    Ok(Chorus {
        rate: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_RATE)?,
        depth: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_DEPTH)?,
        offset: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_OFFSET)?,
    })
}

fn load_modulation_fx_phaser(default_params_node: &Element) -> Result<Phaser, SerializationError> {
    Ok(Phaser {
        rate: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_RATE)?,
        depth: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_DEPTH)?,
        feedback: xml::parse_children_element_content(default_params_node, keys::MODULATION_FX_FEEDBACK)?,
    })
}

pub(crate) fn load_patch_cables(root: &Element) -> Result<Vec<PatchCable>, SerializationError> {
    let cables = xml::get_all_children_element_with_name(root, keys::PATCH_CABLE);
    let mut patch_cables = Vec::new();

    for cable in cables {
        patch_cables.push(load_patch_cable(cable)?);
    }

    Ok(patch_cables)
}

fn load_mod_knob(element: &Element) -> Result<ModKnob, SerializationError> {
    Ok(ModKnob {
        control_param: xml::parse_children_element_content(element, keys::MOD_KNOB_CONTROL_PARAM)?,
        patch_amount_from_source: xml::parse_opt_children_element_content(element, keys::MOD_KNOB_PATCH_AMOUNT_FROM_SOURCE)?,
    })
}

pub(crate) fn load_mod_knobs(root: &Element) -> Result<Vec<ModKnob>, SerializationError> {
    let mod_knob_nodes = xml::get_all_children_element_with_name(root, keys::MOD_KNOB);
    let mut mod_knobs = Vec::new();

    for mod_knob_node in mod_knob_nodes {
        mod_knobs.push(load_mod_knob(mod_knob_node)?);
    }

    Ok(mod_knobs)
}

fn load_patch_cable(root: &Element) -> Result<PatchCable, SerializationError> {
    Ok(PatchCable {
        source: xml::parse_children_element_content(root, keys::PATCH_CABLE_SOURCE)?,
        destination: xml::parse_children_element_content(root, keys::PATCH_CABLE_DESTINATION)?,
        amount: xml::parse_children_element_content(root, keys::PATCH_CABLE_AMOUNT)?,
    })
}

pub(crate) fn load_global_lpf(kit_node: &Element) -> Result<Lpf, SerializationError> {
    let default_params_node = xml::get_children_element(kit_node, keys::DEFAULT_PARAMS)?;
    let default_lpf_node = xml::get_children_element(default_params_node, keys::LPF)?;

    Ok(Lpf {
        frequency: xml::parse_children_element_content(default_lpf_node, keys::FREQUENCY)?,
        resonance: xml::parse_children_element_content(default_lpf_node, keys::RESONANCE)?,
    })
}

pub(crate) fn load_global_hpf(kit_node: &Element) -> Result<Hpf, SerializationError> {
    let default_params_node = xml::get_children_element(kit_node, keys::DEFAULT_PARAMS)?;
    let default_lpf_node = xml::get_children_element(default_params_node, keys::HPF)?;

    Ok(Hpf {
        frequency: xml::parse_children_element_content(default_lpf_node, keys::FREQUENCY)?,
        resonance: xml::parse_children_element_content(default_lpf_node, keys::RESONANCE)?,
    })
}

pub(crate) fn load_global_equalizer(kit_node: &Element) -> Result<Equalizer, SerializationError> {
    Ok(match xml::get_opt_children_element(kit_node, keys::DEFAULT_PARAMS) {
        Some(default_params_node) => load_equalizer(xml::get_children_element(default_params_node, keys::EQUALIZER)?)?,
        None => Equalizer::default(),
    })
}

pub(crate) fn load_global_hexu(kit_node: &Element, key: &str) -> Result<HexU50, SerializationError> {
    Ok(match xml::get_opt_children_element(kit_node, keys::DEFAULT_PARAMS) {
        Some(default_params_node) => xml::parse_children_element_content(default_params_node, key)?,
        None => 0.into(),
    })
}

pub(crate) fn load_global_pan(kit_node: &Element) -> Result<Pan, SerializationError> {
    Ok(match xml::get_opt_children_element(kit_node, keys::DEFAULT_PARAMS) {
        Some(default_params_node) => xml::parse_children_element_content(default_params_node, keys::PAN)?,
        None => Pan::default(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        deserialize_synth, serialize_synth,
        values::{
            AttackSidechain, ClippingAmount, FineTranspose, LfoShape, LpfMode, Pan, Polyphony, ReleaseSidechain, RetrigPhase,
            Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
        },
    };

    use super::*;

    #[test]
    fn load_valid_kit_xml() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/KITS/KIT026.XML")).unwrap();
        let kit = load_kit_nodes(&xml_elements);

        assert!(kit.is_ok());
    }

    #[test]
    fn load_save_load_sound_subtractive() {
        let synth = deserialize_synth(include_str!("../../data_tests/SYNTHS/SYNT061.XML")).unwrap();
        let xml = serialize_synth(&synth).unwrap();
        let reloaded_synth = deserialize_synth(&xml).unwrap();

        assert_eq!(reloaded_synth, synth);
    }

    #[test]
    fn load_valid_sound_subtractive() {
        let xml_elements = xml::load_xml(include_str!("../../data_tests/SYNTHS/SYNT061.XML")).unwrap();
        let synth = load_synth_nodes(&xml_elements).unwrap();
        let sound = &synth.sound;

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

        assert_eq!(sound.sidechain.sync, SyncLevel::Sixteenth);
        assert_eq!(sound.sidechain.attack, AttackSidechain::try_from(327244).unwrap());
        assert_eq!(sound.sidechain.release, ReleaseSidechain::try_from(936).unwrap());

        assert_eq!(sound.equalizer.bass_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.bass_frequency, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_level, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.equalizer.treble_frequency, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.arpeggiator.mode, ArpeggiatorMode::Off);
        assert_eq!(sound.arpeggiator.octaves_count, 2.into());
        assert_eq!(sound.arpeggiator.gate, HexU50::parse("0x00000000").unwrap());
        assert_eq!(sound.arpeggiator.rate, HexU50::parse("0x00000000").unwrap());

        assert_eq!(sound.delay.amount, HexU50::parse("0x80000000").unwrap());
        assert_eq!(sound.delay.rate, HexU50::parse("0xFFFFFFE9").unwrap());
        assert_eq!(sound.delay.ping_pong, OnOff::On);
        assert_eq!(sound.delay.analog, OnOff::Off);
        assert_eq!(sound.delay.sync_level, SyncLevel::Eighth);

        let generator = sound
            .generator
            .as_subtractive()
            .unwrap();

        assert_eq!(generator.lpf_mode, LpfMode::Lpf24);
        assert_eq!(generator.lpf_frequency, HexU50::parse("0x24000000").unwrap());
        assert_eq!(generator.lpf_resonance, HexU50::parse("0x82000000").unwrap());
        assert_eq!(generator.hpf_frequency, HexU50::parse("0x1C000000").unwrap());
        assert_eq!(generator.hpf_resonance, HexU50::parse("0x80000000").unwrap());
        assert_eq!(generator.osc1_volume, HexU50::parse("0x70A3D6DF").unwrap());
        assert_eq!(generator.osc2_volume, HexU50::parse("0x7FFFFFD2").unwrap());
        assert_eq!(generator.osc2_sync, OnOff::Off);

        let waveform = generator.osc1.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Square);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(0));
        assert_eq!(waveform.retrig_phase, RetrigPhase::new(0));
        assert_eq!(waveform.pulse_width, HexU50::parse("0x00000000").unwrap());

        let waveform = generator.osc2.as_waveform().unwrap();

        assert_eq!(waveform.osc_type, OscType::Saw);
        assert_eq!(waveform.transpose, Transpose::new(0));
        assert_eq!(waveform.fine_transpose, FineTranspose::new(8));
        assert_eq!(waveform.retrig_phase, RetrigPhase::new(0));
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

        assert_eq!(generator.osc1_volume, HexU50::parse("0x7FFFFFFF").unwrap());
        assert_eq!(generator.osc2_volume, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.osc1.transpose, Transpose::new(0));
        assert_eq!(generator.osc1.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.osc1.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.osc1.feedback, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.osc2.transpose, Transpose::new(0));
        assert_eq!(generator.osc2.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.osc2.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.osc2.feedback, HexU50::parse("0x80000000").unwrap());

        assert_eq!(generator.modulator1.transpose, Transpose::new(-15));
        assert_eq!(generator.modulator1.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.modulator1.retrig_phase, RetrigPhase::new(0));
        assert_eq!(generator.modulator2_to_modulator1, OnOff::Off);

        assert_eq!(generator.modulator2.transpose, Transpose::new(-12));
        assert_eq!(generator.modulator2.fine_transpose, FineTranspose::new(0));
        assert_eq!(generator.modulator2.retrig_phase, RetrigPhase::new(0));
    }
}
