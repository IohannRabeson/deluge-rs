use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    serialization::{
        default_params::{DefaultParamsMut, TwinSelector},
        keys,
        serialization_common::LATEST_SUPPORTED_FIRMWARE_VERSION,
        xml,
    },
    values::*,
    Arpeggiator, Chorus, CvGateOutput, Delay, Distorsion, Envelope, Equalizer, Flanger, FmCarrier, FmGenerator, FmModulator, Kit,
    Lfo1, Lfo2, MidiOutput, ModKnob, ModulationFx, Oscillator, PatchCable, Phaser, RingModGenerator, RowKit, Sample,
    SampleOneZone, SampleOscillator, SampleRange, SampleZone, SerializationError, Sidechain, Sound, SoundGenerator,
    SubtractiveGenerator, Synth, Unison, WaveformOscillator, Lpf, Hpf,
};

use xmltree::Element;

pub fn write_synth(synth: &Synth) -> Result<Element, SerializationError> {
    let mut sound_node = write_sound(&synth.sound, None)?;

    xml::insert_attribute(&mut sound_node, keys::FIRMWARE_VERSION, &LATEST_SUPPORTED_FIRMWARE_VERSION)?;
    xml::insert_attribute(
        &mut sound_node,
        keys::EARLIEST_COMPATIBLE_FIRMWARE,
        &LATEST_SUPPORTED_FIRMWARE_VERSION,
    )?;

    Ok(sound_node)
}

pub fn write_kit(kit: &Kit) -> Result<Element, SerializationError> {
    let mut kit_node = Element::new(keys::KIT);

    xml::insert_attribute(&mut kit_node, keys::FIRMWARE_VERSION, &LATEST_SUPPORTED_FIRMWARE_VERSION)?;
    xml::insert_attribute(
        &mut kit_node,
        keys::EARLIEST_COMPATIBLE_FIRMWARE,
        &LATEST_SUPPORTED_FIRMWARE_VERSION,
    )?;

    xml::insert_attribute(&mut kit_node, keys::LPF_MODE, &kit.lpf_mode)?;
    xml::insert_attribute(&mut kit_node, keys::CURRENT_FILTER_TYPE, &kit.current_filter_type)?;

    let mut default_params_node = Rc::new(RefCell::new(Element::new(keys::DEFAULT_PARAMS)));
    let default_delay_node = Rc::new(RefCell::new(Element::new(keys::DELAY)));
    xml::insert_child(&mut kit_node, write_global_delay(&kit.delay, &default_delay_node)?)?;
    xml::insert_child(&mut kit_node, write_global_sidechain(&kit.sidechain, &default_params_node)?)?;
    
    write_modulation_fx(&kit.modulation_fx, &mut kit_node, &default_params_node)?;

    xml::insert_child(&mut kit_node, write_sound_sources(&kit.rows)?)?;

    if let Some(index) = kit.selected_drum_index {
        xml::insert_child(&mut kit_node, write_selected_drum_index(index)?)?;
    }

    // Must be done at the end to ensure 'default_params_node' has all his children added.
    xml::insert_attribute_rc(&mut default_params_node, keys::BIT_CRUSH, &kit.bit_crush)?;
    xml::insert_attribute_rc(&mut default_params_node, keys::DECIMATION, &kit.decimation)?;
    xml::insert_child_rc(&mut default_params_node, write_global_lpf(&kit.lpf)?);
    xml::insert_child_rc(&mut default_params_node, write_global_hpf(&kit.hpf)?);
    xml::insert_child_rc(&mut default_params_node, write_equalizer(&kit.equalizer)?);
    xml::insert_child(&mut default_params_node.borrow_mut(), default_delay_node.borrow().clone())?;
    xml::insert_child(&mut kit_node, default_params_node.borrow().clone())?;

    Ok(kit_node)
}

fn write_sound_sources(rows: &[RowKit]) -> Result<Element, SerializationError> {
    let mut sound_source_node = Element::new(keys::SOUND_SOURCES);

    for row in rows {
        let node = match row {
            RowKit::AudioOutput(sound) => write_sound(&sound.sound, Some(&sound.name))?,
            RowKit::CvGateOutput(gate) => write_gate_output(gate)?,
            RowKit::MidiOutput(midi) => write_midi_output(midi)?,
        };

        xml::insert_child(&mut sound_source_node, node)?;
    }
    Ok(sound_source_node)
}

fn write_selected_drum_index(index: u32) -> Result<Element, SerializationError> {
    let mut selected_drum_index_node = Element::new(keys::SELECTED_DRUM_INDEX);

    selected_drum_index_node
        .children
        .push(xmltree::XMLNode::Text(index.to_string()));

    Ok(selected_drum_index_node)
}

fn write_gate_output(gate: &CvGateOutput) -> Result<Element, SerializationError> {
    let mut gate_output_node = Element::new(keys::GATE_OUTPUT);

    xml::insert_attribute(&mut gate_output_node, keys::CHANNEL, &gate.channel)?;

    Ok(gate_output_node)
}

fn write_midi_output(midi_output: &MidiOutput) -> Result<Element, SerializationError> {
    let mut midi_output_node = Element::new(keys::MIDI_OUTPUT);

    xml::insert_attribute(&mut midi_output_node, keys::CHANNEL, &midi_output.channel)?;
    xml::insert_attribute(&mut midi_output_node, keys::NOTE, &midi_output.note)?;

    Ok(midi_output_node)
}

fn write_sound(sound: &Sound, name: Option<&String>) -> Result<Element, SerializationError> {
    let mut sound_node = Element::new(keys::SOUND);
    let default_params_node = Rc::new(RefCell::new(Element::new(keys::DEFAULT_PARAMS)));

    if let Some(name) = name {
        if !name.is_empty() {
            xml::insert_attribute(&mut sound_node, keys::NAME, name)?;
        }
    }

    xml::insert_attribute(&mut sound_node, keys::MODE, &sound.generator.to_sound_type())?;
    xml::insert_attribute(&mut sound_node, keys::POLYPHONIC, &sound.polyphonic)?;
    xml::insert_opt_attribute(&mut sound_node, keys::SIDECHAIN_SEND, &sound.sidechain_send)?;
    xml::insert_attribute(&mut sound_node, keys::VOICE_PRIORITY, &sound.voice_priority)?;
    xml::insert_attribute_rc(&default_params_node, keys::VOLUME, &sound.volume)?;
    xml::insert_attribute_rc(&default_params_node, keys::REVERB_AMOUNT, &sound.reverb_amount)?;
    xml::insert_attribute_rc(&default_params_node, keys::STUTTER_RATE, &sound.stutter_rate)?;
    xml::insert_attribute_rc(&default_params_node, keys::PAN, &sound.pan)?;
    xml::insert_attribute_rc(&default_params_node, keys::PORTAMENTO, &sound.portamento)?;

    match &sound.generator {
        SoundGenerator::Subtractive(ref generator) => write_subtractive_sound(generator, &mut sound_node, &default_params_node)?,
        SoundGenerator::Fm(generator) => write_fm_sound(generator, &mut sound_node, &default_params_node)?,
        SoundGenerator::RingMod(generator) => write_ringmod_sound(generator, &mut sound_node, &default_params_node)?,
    }

    xml::insert_child_rc(&default_params_node, write_envelope(&sound.envelope1, TwinSelector::A)?);
    xml::insert_child_rc(&default_params_node, write_envelope(&sound.envelope2, TwinSelector::B)?);
    xml::insert_child_rc(&default_params_node, write_equalizer(&sound.equalizer)?);
    xml::insert_child_rc(&default_params_node, write_cables(&sound.cables)?);
    xml::insert_child(&mut sound_node, write_unison(&sound.unison)?)?;
    xml::insert_child(&mut sound_node, write_lfo1(&sound.lfo1, &default_params_node)?)?;
    xml::insert_child(&mut sound_node, write_lfo2(&sound.lfo2, &default_params_node)?)?;
    xml::insert_child(&mut sound_node, write_arpegiator(&sound.arpeggiator, &default_params_node)?)?;
    xml::insert_child(&mut sound_node, write_delay(&sound.delay, &default_params_node)?)?;
    xml::insert_child(&mut sound_node, write_sidechain(&sound.sidechain, &default_params_node)?)?;
    xml::insert_child(&mut sound_node, write_mod_knobs(&sound.mod_knobs)?)?;

    write_modulation_fx(&sound.modulation_fx, &mut sound_node, &default_params_node)?;
    write_distorsion(&sound.distorsion, &mut sound_node, &default_params_node)?;

    // Must be done at the end to ensure 'default_params_node' has all his children added.
    xml::insert_child(&mut sound_node, default_params_node.borrow().clone())?;

    Ok(sound_node)
}

fn write_modulation_fx(
    modulation_fx: &ModulationFx,
    sound_node: &mut Element,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<(), SerializationError> {
    match modulation_fx {
        ModulationFx::Off => {
            xml::insert_attribute(sound_node, keys::MOD_FX_TYPE, &keys::MODULATION_FX_OFF)?;
            xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_RATE, &HexU50::new(25))?;
            xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_FEEDBACK, &HexU50::new(25))?;

            Ok(())
        }
        ModulationFx::Flanger(flanger) => {
            xml::insert_attribute(sound_node, keys::MOD_FX_TYPE, &keys::MODULATION_FX_FLANGER)?;

            write_flanger(flanger, default_params_node)
        }
        ModulationFx::Chorus(chorus) => {
            xml::insert_attribute(sound_node, keys::MOD_FX_TYPE, &keys::MODULATION_FX_CHORUS)?;

            write_chorus(chorus, default_params_node)
        }
        ModulationFx::Phaser(phaser) => {
            xml::insert_attribute(sound_node, keys::MOD_FX_TYPE, &keys::MODULATION_FX_PHASER)?;

            write_phaser(phaser, default_params_node)
        }
    }
}

fn write_phaser(phaser: &Phaser, default_params_node: &Rc<RefCell<Element>>) -> Result<(), SerializationError> {
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_RATE, &phaser.rate)?;
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_FEEDBACK, &phaser.feedback)?;
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_DEPTH, &phaser.depth)?;
    Ok(())
}

fn write_chorus(chorus: &Chorus, default_params_node: &Rc<RefCell<Element>>) -> Result<(), SerializationError> {
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_RATE, &chorus.rate)?;
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_DEPTH, &chorus.depth)?;
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_OFFSET, &chorus.offset)?;

    Ok(())
}

fn write_flanger(flanger: &Flanger, default_params_node: &Rc<RefCell<Element>>) -> Result<(), SerializationError> {
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_RATE, &flanger.rate)?;
    xml::insert_attribute_rc(default_params_node, keys::MODULATION_FX_FEEDBACK, &flanger.feedback)?;

    Ok(())
}

fn write_arpegiator(
    arpeggiator: &Arpeggiator,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<Element, SerializationError> {
    let mut arpegiator_node = Element::new(keys::ARPEGGIATOR);

    xml::insert_attribute(&mut arpegiator_node, keys::ARPEGGIATOR_MODE, &arpeggiator.mode)?;
    xml::insert_attribute(&mut arpegiator_node, keys::SYNC_LEVEL, &arpeggiator.sync_level)?;
    xml::insert_attribute(
        &mut arpegiator_node,
        keys::ARPEGGIATOR_OCTAVE_COUNT,
        &arpeggiator.octaves_count,
    )?;
    xml::insert_attribute_rc(default_params_node, keys::ARPEGGIATOR_RATE, &arpeggiator.rate)?;
    xml::insert_attribute_rc(default_params_node, keys::ARPEGGIATOR_GATE, &arpeggiator.gate)?;

    Ok(arpegiator_node)
}

fn write_lfo1(lfo: &Lfo1, default_params_node: &Rc<RefCell<Element>>) -> Result<Element, SerializationError> {
    let mut lfo_node = Element::new(keys::LFO1);

    xml::insert_attribute(&mut lfo_node, keys::LFO_SHAPE, &lfo.shape)?;
    xml::insert_attribute(&mut lfo_node, keys::SYNC_LEVEL, &lfo.sync_level)?;
    xml::insert_attribute_rc(default_params_node, keys::LFO1_RATE, &lfo.rate)?;

    Ok(lfo_node)
}

fn write_lfo2(lfo: &Lfo2, default_params_node: &Rc<RefCell<Element>>) -> Result<Element, SerializationError> {
    let mut lfo_node = Element::new(keys::LFO2);

    xml::insert_attribute(&mut lfo_node, keys::LFO_SHAPE, &lfo.shape)?;
    xml::insert_attribute_rc(default_params_node, keys::LFO2_RATE, &lfo.rate)?;

    Ok(lfo_node)
}

fn write_subtractive_sound(
    generator: &SubtractiveGenerator,
    sound_node: &mut Element,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<(), SerializationError> {
    let default_params_a = DefaultParamsMut::new(TwinSelector::A, default_params_node.clone());
    let default_params_b = DefaultParamsMut::new(TwinSelector::B, default_params_node.clone());

    let mut osc2_node = write_oscillator(&generator.osc2, &default_params_b)?;

    xml::insert_attribute(&mut osc2_node, keys::OSCILLATOR_SYNC, &generator.osc2_sync)?;

    xml::insert_child(sound_node, write_oscillator(&generator.osc1, &default_params_a)?)?;
    xml::insert_child(sound_node, osc2_node)?;

    xml::insert_attribute_rc(default_params_node, keys::NOISE_VOLUME, &generator.noise)?;
    xml::insert_attribute_rc(default_params_node, keys::LPF_FREQUENCY, &generator.lpf_frequency)?;
    xml::insert_attribute_rc(default_params_node, keys::LPF_RESONANCE, &generator.lpf_resonance)?;
    xml::insert_attribute_rc(default_params_node, keys::HPF_FREQUENCY, &generator.hpf_frequency)?;
    xml::insert_attribute_rc(default_params_node, keys::HPF_RESONANCE, &generator.hpf_resonance)?;
    xml::insert_attribute(sound_node, keys::LPF_MODE, &generator.lpf_mode)?;

    Ok(())
}

fn write_oscillator(osc: &Oscillator, default_params: &DefaultParamsMut) -> Result<Element, SerializationError> {
    Ok(match &osc {
        Oscillator::Waveform(oscillator) => write_waveform_oscillator(oscillator, default_params)?,
        Oscillator::Sample(oscillator) => write_sample_oscillator(oscillator, default_params)?,
    })
}

fn write_carrier(osc: &FmCarrier, default_params: &DefaultParamsMut) -> Result<Element, SerializationError> {
    let mut node = default_params.create_element(keys::OSC1, keys::OSC2);

    xml::insert_attribute(&mut node, keys::TRANSPOSE, &osc.transpose)?;
    xml::insert_attribute(&mut node, keys::CENTS, &osc.fine_transpose)?;
    xml::insert_attribute(&mut node, keys::RETRIG_PHASE, &osc.retrig_phase)?;
    default_params.insert_attribute(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B, &osc.volume)?;
    default_params.insert_attribute(keys::FEEDBACK_CARRIER1, keys::FEEDBACK_CARRIER2, &osc.feedback)?;

    Ok(node)
}

fn write_modulator(modulator: &FmModulator, default_params: &DefaultParamsMut) -> Result<Element, SerializationError> {
    let mut node = default_params.create_element(keys::FM_MODULATOR1, keys::FM_MODULATOR2);

    xml::insert_attribute(&mut node, keys::TRANSPOSE, &modulator.transpose)?;
    xml::insert_attribute(&mut node, keys::CENTS, &modulator.fine_transpose)?;
    xml::insert_attribute(&mut node, keys::RETRIG_PHASE, &modulator.retrig_phase)?;
    default_params.insert_attribute(keys::AMOUNT_MODULATOR1, keys::AMOUNT_MODULATOR2, &modulator.amount)?;
    default_params.insert_attribute(keys::FEEDBACK_MODULATOR1, keys::FEEDBACK_MODULATOR2, &modulator.feedback)?;

    Ok(node)
}

fn write_sample_oscillator(sample: &SampleOscillator, default_params: &DefaultParamsMut) -> Result<Element, SerializationError> {
    let mut node = default_params.create_element(keys::OSC1, keys::OSC2);

    xml::insert_attribute(&mut node, keys::TYPE, &OscType::Sample)?;
    xml::insert_attribute(&mut node, keys::TRANSPOSE, &sample.transpose)?;
    xml::insert_attribute(&mut node, keys::CENTS, &sample.fine_transpose)?;
    xml::insert_attribute(&mut node, keys::REVERSED, &sample.reversed)?;
    xml::insert_attribute(&mut node, keys::LOOP_MODE, &sample.mode)?;
    xml::insert_attribute(&mut node, keys::TIME_STRETCH_ENABLE, &sample.pitch_speed)?;
    xml::insert_attribute(&mut node, keys::TIME_STRETCH_AMOUNT, &sample.time_stretch_amount)?;
    xml::insert_attribute(&mut node, keys::LINEAR_INTERPOLATION, &sample.linear_interpolation)?;

    write_sample(&mut node, &sample.sample)?;

    default_params.insert_attribute(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B, &sample.volume)?;

    Ok(node)
}

fn write_sample(node: &mut Element, sample: &Sample) -> Result<(), SerializationError> {
    match sample {
        Sample::OneZone(one_zone) => write_sample_one_zone(node, one_zone),
        Sample::SampleRanges(ranges) => write_sample_ranges(node, ranges),
    }
}

fn write_sample_ranges(node: &mut Element, ranges: &[SampleRange]) -> Result<(), SerializationError> {
    let mut sample_ranges_node = Element::new(keys::SAMPLE_RANGES);

    for sample_range in ranges {
        let mut sample_range_node = Element::new(keys::SAMPLE_RANGE);
        let zone_node = write_sample_zone(&sample_range.zone)?;

        xml::insert_opt_attribute(
            &mut sample_range_node,
            keys::SAMPLE_RANGE_TOP_NOTE,
            &sample_range.range_top_note,
        )?;
        xml::insert_opt_attribute_if_not_default(&mut sample_range_node, keys::TRANSPOSE, &sample_range.transpose)?;
        xml::insert_opt_attribute_if_not_default(&mut sample_range_node, keys::CENTS, &sample_range.fine_transpose)?;

        xml::insert_attribute(&mut sample_range_node, keys::FILE_NAME, &sample_range.file_path)?;
        xml::insert_child(&mut sample_range_node, zone_node)?;
        xml::insert_child(&mut sample_ranges_node, sample_range_node)?;
    }

    xml::insert_child(node, sample_ranges_node)?;

    Ok(())
}

fn write_sample_one_zone(node: &mut Element, sample: &SampleOneZone) -> Result<(), SerializationError> {
    xml::insert_attribute(node, keys::FILE_NAME, &sample.file_path)?;

    if let Some(zone) = &sample.zone {
        xml::insert_child(node, write_sample_zone(zone)?)?;
    }

    Ok(())
}

fn write_sample_zone(zone: &SampleZone) -> Result<Element, SerializationError> {
    let mut sample_zone_node = Element::new(keys::ZONE);

    xml::insert_attribute(&mut sample_zone_node, keys::START_SAMPLES_POS, &zone.start)?;
    xml::insert_attribute(&mut sample_zone_node, keys::END_SAMPLES_POS, &zone.end)?;
    xml::insert_opt_attribute(&mut sample_zone_node, keys::START_LOOP_SAMPLES_POS, &zone.start_loop)?;
    xml::insert_opt_attribute(&mut sample_zone_node, keys::END_LOOP_SAMPLES_POS, &zone.end_loop)?;

    Ok(sample_zone_node)
}

fn write_waveform_oscillator(
    oscillator: &WaveformOscillator,
    default_params: &DefaultParamsMut,
) -> Result<Element, SerializationError> {
    let mut node = default_params.create_element(keys::OSC1, keys::OSC2);

    xml::insert_attribute(&mut node, keys::TYPE, &oscillator.osc_type)?;
    xml::insert_attribute(&mut node, keys::TRANSPOSE, &oscillator.transpose)?;
    xml::insert_attribute(&mut node, keys::CENTS, &oscillator.fine_transpose)?;
    xml::insert_attribute(&mut node, keys::RETRIG_PHASE, &oscillator.retrig_phase)?;
    default_params.insert_attribute(keys::VOLUME_OSC_A, keys::VOLUME_OSC_B, &oscillator.volume)?;
    default_params.insert_attribute(keys::PULSE_WIDTH_OSC_A, keys::PULSE_WIDTH_OSC_B, &oscillator.pulse_width)?;

    Ok(node)
}

fn write_fm_sound(
    generator: &FmGenerator,
    sound_node: &mut Element,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<(), SerializationError> {
    let default_params_a = DefaultParamsMut::new(TwinSelector::A, default_params_node.clone());
    let default_params_b = DefaultParamsMut::new(TwinSelector::B, default_params_node.clone());
    let mut mod2_node = write_modulator(&generator.modulator2, &default_params_b)?;

    xml::insert_child(sound_node, write_carrier(&generator.osc1, &default_params_a)?)?;
    xml::insert_child(sound_node, write_carrier(&generator.osc2, &default_params_b)?)?;
    xml::insert_child(sound_node, write_modulator(&generator.modulator1, &default_params_a)?)?;
    xml::insert_attribute(&mut mod2_node, keys::FM_MOD1_TO_MOD2, &generator.modulator2_to_modulator1)?;
    xml::insert_child(sound_node, mod2_node)?;

    Ok(())
}

fn write_ringmod_sound(
    generator: &RingModGenerator,
    sound_node: &mut Element,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<(), SerializationError> {
    let default_params_a = DefaultParamsMut::new(TwinSelector::A, default_params_node.clone());
    let default_params_b = DefaultParamsMut::new(TwinSelector::B, default_params_node.clone());
    let mut osc2_node = write_oscillator(&generator.osc2, &default_params_b)?;

    xml::insert_attribute(&mut osc2_node, keys::OSCILLATOR_SYNC, &generator.osc2_sync)?;
    xml::insert_child(sound_node, write_oscillator(&generator.osc1, &default_params_a)?)?;
    xml::insert_child(sound_node, osc2_node)?;
    xml::insert_attribute_rc(default_params_node, keys::NOISE_VOLUME, &generator.noise)?;

    Ok(())
}

fn write_envelope(envelope: &Envelope, selector: TwinSelector) -> Result<Element, SerializationError> {
    let mut node = Element::new(selector.get_key(keys::ENVELOPE1, keys::ENVELOPE2));

    xml::insert_attribute(&mut node, keys::ENV_ATTACK, &envelope.attack)?;
    xml::insert_attribute(&mut node, keys::ENV_DECAY, &envelope.decay)?;
    xml::insert_attribute(&mut node, keys::ENV_SUSTAIN, &envelope.sustain)?;
    xml::insert_attribute(&mut node, keys::ENV_RELEASE, &envelope.release)?;

    Ok(node)
}

fn write_equalizer(equalizer: &Equalizer) -> Result<Element, SerializationError> {
    let mut equalizer_node = Element::new(keys::EQUALIZER);

    xml::insert_attribute(&mut equalizer_node, keys::EQ_BASS, &equalizer.bass_level)?;
    xml::insert_attribute(&mut equalizer_node, keys::EQ_BASS_FREQUENCY, &equalizer.bass_frequency)?;
    xml::insert_attribute(&mut equalizer_node, keys::EQ_TREBLE, &equalizer.treble_level)?;
    xml::insert_attribute(&mut equalizer_node, keys::EQ_TREBLE_FREQUENCY, &equalizer.treble_frequency)?;

    Ok(equalizer_node)
}

fn write_unison(unison: &Unison) -> Result<Element, SerializationError> {
    let mut unison_node = Element::new(keys::UNISON);

    xml::insert_attribute(&mut unison_node, keys::UNISON_VOICE_COUNT, &unison.voice_count)?;
    xml::insert_attribute(&mut unison_node, keys::UNISON_DETUNE, &unison.detune)?;

    Ok(unison_node)
}

fn write_distorsion(
    distorsion: &Distorsion,
    sound_node: &mut Element,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<(), SerializationError> {
    xml::insert_attribute(sound_node, keys::CLIPPING_AMOUNT, &distorsion.saturation)?;
    xml::insert_attribute_rc(default_params_node, keys::BIT_CRUSH, &distorsion.bit_crush)?;
    xml::insert_attribute_rc(default_params_node, keys::DECIMATION, &distorsion.decimation)?;

    Ok(())
}

fn write_delay(delay: &Delay, default_params_node: &Rc<RefCell<Element>>) -> Result<Element, SerializationError> {
    let mut delay_node = Element::new(keys::DELAY);

    xml::insert_attribute(&mut delay_node, keys::PING_PONG, &delay.ping_pong)?;
    xml::insert_attribute(&mut delay_node, keys::ANALOG, &delay.analog)?;
    xml::insert_attribute(&mut delay_node, keys::SYNC_LEVEL, &delay.sync_level)?;
    xml::insert_attribute_rc(default_params_node, keys::DELAY_FEEDBACK, &delay.amount)?;
    xml::insert_attribute_rc(default_params_node, keys::DELAY_RATE, &delay.rate)?;

    Ok(delay_node)
}

fn write_global_delay(delay: &Delay, default_params_node: &Rc<RefCell<Element>>) -> Result<Element, SerializationError> {
    let mut delay_node = Element::new(keys::DELAY);

    xml::insert_attribute(&mut delay_node, keys::PING_PONG, &delay.ping_pong)?;
    xml::insert_attribute(&mut delay_node, keys::ANALOG, &delay.analog)?;
    xml::insert_attribute(&mut delay_node, keys::SYNC_LEVEL, &delay.sync_level)?;
    xml::insert_attribute_rc(default_params_node, keys::FEEDBACK, &delay.amount)?;
    xml::insert_attribute_rc(default_params_node, keys::RATE, &delay.rate)?;

    Ok(delay_node)
}

fn write_sidechain(sidechain: &Sidechain, default_params_node: &Rc<RefCell<Element>>) -> Result<Element, SerializationError> {
    let mut sidechain_node = Element::new(keys::COMPRESSOR);

    xml::insert_attribute(&mut sidechain_node, keys::COMPRESSOR_ATTACK, &sidechain.attack)?;
    xml::insert_attribute(&mut sidechain_node, keys::COMPRESSOR_RELEASE, &sidechain.release)?;
    xml::insert_attribute(&mut sidechain_node, keys::COMPRESSOR_SYNCLEVEL, &sidechain.sync)?;
    xml::insert_attribute_rc(default_params_node, keys::COMPRESSOR_SHAPE, &sidechain.shape)?;

    Ok(sidechain_node)
}

fn write_global_sidechain(
    sidechain: &Sidechain,
    default_params_node: &Rc<RefCell<Element>>,
) -> Result<Element, SerializationError> {
    let mut sidechain_node = Element::new(keys::COMPRESSOR);

    xml::insert_attribute(&mut sidechain_node, keys::COMPRESSOR_ATTACK, &sidechain.attack)?;
    xml::insert_attribute(&mut sidechain_node, keys::COMPRESSOR_RELEASE, &sidechain.release)?;
    xml::insert_attribute(&mut sidechain_node, keys::COMPRESSOR_SYNCLEVEL, &sidechain.sync)?;
    xml::insert_attribute_rc(default_params_node, keys::SIDECHAIN_COMPRESSOR_SHAPE, &sidechain.shape)?;

    Ok(sidechain_node)
}

fn write_global_lpf(lpf: &Lpf) -> Result<Element, SerializationError> {
    let mut lpf_node = Element::new(keys::LPF);

    xml::insert_attribute(&mut lpf_node, keys::FREQUENCY, &lpf.frequency)?;
    xml::insert_attribute(&mut lpf_node, keys::RESONANCE, &lpf.resonance)?;

    Ok(lpf_node)
}

fn write_global_hpf(hpf: &Hpf) -> Result<Element, SerializationError> {
    let mut hpf_node = Element::new(keys::HPF);

    xml::insert_attribute(&mut hpf_node, keys::FREQUENCY, &hpf.frequency)?;
    xml::insert_attribute(&mut hpf_node, keys::RESONANCE, &hpf.resonance)?;

    Ok(hpf_node)
}

fn write_cables(patch_cables: &[PatchCable]) -> Result<Element, SerializationError> {
    let mut cables_node = Element::new(keys::PATCH_CABLES);

    for cable in patch_cables {
        xml::insert_child(&mut cables_node, write_cable(cable)?)?;
    }

    Ok(cables_node)
}

fn write_cable(cable: &PatchCable) -> Result<Element, SerializationError> {
    let mut cable_node = Element::new(keys::PATCH_CABLE);

    xml::insert_attribute(&mut cable_node, keys::PATCH_CABLE_SOURCE, &cable.source)?;
    xml::insert_attribute(&mut cable_node, keys::PATCH_CABLE_DESTINATION, &cable.destination)?;
    xml::insert_attribute(&mut cable_node, keys::PATCH_CABLE_AMOUNT, &cable.amount)?;

    Ok(cable_node)
}

fn write_mod_knobs(mod_knobs: &[ModKnob]) -> Result<Element, SerializationError> {
    let mut mod_knobs_node = Element::new(keys::MOD_KNOBS);

    for mod_knob in mod_knobs {
        let mut mod_knob_node = Element::new(keys::MOD_KNOB);

        xml::insert_attribute(&mut mod_knob_node, keys::MOD_KNOB_CONTROL_PARAM, &mod_knob.control_param)?;
        if let Some(patch_amount_from_source) = &mod_knob.patch_amount_from_source {
            xml::insert_attribute(
                &mut mod_knob_node,
                keys::MOD_KNOB_PATCH_AMOUNT_FROM_SOURCE,
                patch_amount_from_source,
            )?;
        }
        xml::insert_child(&mut mod_knobs_node, mod_knob_node)?;
    }

    Ok(mod_knobs_node)
}
