use crate::{
    values::{AttackSidechain, OnOff, ReleaseSidechain, SoundType, TableIndex},
    Arpeggiator, Delay, Kit, RowKit, SerializationError, Sidechain, Sound, SoundGenerator, SubtractiveGenerator, Synth,
};
use xmltree::Element;

use super::{
    default_params::{DefaultParams, TwinSelector},
    keys,
    serialization_v1::{
        load_distorsion, load_envelope, load_equalizer, load_fm_sound, load_global_equalizer, load_global_hexu, load_global_hpf,
        load_global_lpf, load_global_pan, load_lfo1, load_lfo2, load_mod_knobs, load_modulation_fx, load_oscillator,
        load_patch_cables, load_ringmode_sound, load_sound_source, load_unison,
    },
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
        return Err(result_with_error.as_ref().unwrap_err().clone());
    }

    return Ok(Kit {
        rows: sources.iter().flatten().cloned().collect::<Vec<RowKit>>(),
        lpf_mode: xml::parse_children_element_content(kit_node, keys::LPF_MODE)?,
        modulation_fx: load_modulation_fx(kit_node)?,
        current_filter_type: xml::parse_children_element_content(kit_node, keys::CURRENT_FILTER_TYPE)?,
        selected_drum_index: xml::parse_children_element_content(kit_node, keys::SELECTED_DRUM_INDEX)?,
        volume: load_global_hexu(kit_node, keys::VOLUME)?,
        reverb_amount: load_global_hexu(kit_node, keys::REVERB_AMOUNT)?,
        pan: load_global_pan(kit_node)?,
        bit_crush: load_global_hexu(kit_node, keys::BIT_CRUSH)?,
        decimation: load_global_hexu(kit_node, keys::DECIMATION)?,
        stutter_rate: load_global_hexu(kit_node, keys::STUTTER_RATE)?,
        delay: load_global_delay(kit_node)?,
        sidechain: load_global_sidechain(kit_node)?,
        lpf: load_global_lpf(kit_node)?,
        hpf: load_global_hpf(kit_node)?,
        equalizer: load_global_equalizer(kit_node)?,
    });
}

fn load_sound(root: &Element) -> Result<Sound, SerializationError> {
    let sound_type = xml::parse_children_element_content::<SoundType>(root, keys::MODE)?;
    let default_params_node = xml::get_children_element(root, keys::DEFAULT_PARAMS)?;

    let generator = match sound_type {
        SoundType::Subtractive => load_subtractive_sound(root)?,
        SoundType::Fm => load_fm_sound(root)?,
        SoundType::RingMod => load_ringmode_sound(root)?,
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

fn load_subtractive_sound(root: &Element) -> Result<SoundGenerator, SerializationError> {
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

fn load_delay(root: &Element, default_params_node: &Element) -> Result<Delay, SerializationError> {
    Ok(Delay {
        ping_pong: xml::parse_children_element_content(root, keys::PING_PONG)?,
        analog: xml::parse_children_element_content(root, keys::ANALOG)?,
        sync_level: xml::parse_children_element_content(root, keys::SYNC_LEVEL)?,
        amount: xml::parse_children_element_content(default_params_node, keys::DELAY_FEEDBACK)?,
        rate: xml::parse_children_element_content(default_params_node, keys::DELAY_RATE)?,
    })
}

/// Loading the global delay is slightly different than loading the "normal" one.
/// The keys for the feedback and rate parameters are different.
fn load_global_delay(kit_node: &Element) -> Result<Delay, SerializationError> {
    let default_params_node = xml::get_children_element(kit_node, keys::DEFAULT_PARAMS)?;
    let default_delay_node = xml::get_children_element(default_params_node, keys::DELAY)?;
    let delay_node = xml::get_children_element(kit_node, keys::DELAY)?;

    Ok(Delay {
        ping_pong: xml::parse_children_element_content(delay_node, keys::PING_PONG)?,
        analog: xml::parse_children_element_content(delay_node, keys::ANALOG)?,
        sync_level: xml::parse_children_element_content(delay_node, keys::SYNC_LEVEL)?,
        amount: xml::parse_children_element_content(default_delay_node, keys::FEEDBACK)?,
        rate: xml::parse_children_element_content(default_delay_node, keys::RATE)?,
    })
}

fn load_arpeggiator(root: &Element, default_params_node: &Element) -> Result<Arpeggiator, SerializationError> {
    Ok(match xml::get_opt_children_element(root, keys::ARPEGGIATOR) {
        Some(arpeggiator_node) => Arpeggiator {
            mode: xml::parse_children_element_content(arpeggiator_node, keys::ARPEGGIATOR_MODE)?,
            sync_level: xml::parse_children_element_content(arpeggiator_node, keys::SYNC_LEVEL)?,
            octaves_count: xml::parse_children_element_content(arpeggiator_node, keys::ARPEGGIATOR_OCTAVE_COUNT)?,
            rate: xml::parse_children_element_content(default_params_node, keys::ARPEGGIATOR_RATE)?,
            gate: xml::parse_children_element_content(default_params_node, keys::ARPEGGIATOR_GATE)?,
        },
        None => Arpeggiator::default(),
    })
}

fn load_sidechain(root: &Element, default_params_node: &Element) -> Result<Sidechain, SerializationError> {
    Ok(Sidechain {
        attack: xml::parse_children_element_content(root, keys::COMPRESSOR_ATTACK)?,
        release: xml::parse_children_element_content(root, keys::COMPRESSOR_RELEASE)?,
        shape: xml::parse_children_element_content(default_params_node, keys::COMPRESSOR_SHAPE)?,
        sync: xml::parse_children_element_content(root, keys::COMPRESSOR_SYNCLEVEL)?,
    })
}

fn load_global_sidechain(kit_node: &Element) -> Result<Sidechain, SerializationError> {
    Ok(match xml::get_opt_children_element(kit_node, keys::COMPRESSOR) {
        Some(compressor_node) => Sidechain {
            attack: AttackSidechain::new(TableIndex::new(7)),
            release: ReleaseSidechain::new(TableIndex::new(28)),
            shape: 18.into(),
            sync: xml::parse_children_element_content(compressor_node, keys::COMPRESSOR_SYNCLEVEL)?,
        },
        None => Sidechain::default(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        load_synth, save_synth,
        values::{
            ArpeggiatorMode, AttackSidechain, ClippingAmount, FineTranspose, HexU50, LfoShape, LpfMode, OscType, Pan, Polyphony,
            ReleaseSidechain, RetrigPhase, SyncLevel, Transpose, UnisonDetune, UnisonVoiceCount, VoicePriority,
        },
        ModulationFx,
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
        assert_eq!(sound.arpeggiator.octaves_count, 2.into());
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
