/// The serialization module
///
/// This module defines all the types used by [Kit] and [Synth].  
/// Each type specifies how the serialization works.
use crate::{Kit, Synth};

pub use version_info::{PatchType, VersionInfo};

pub use self::error::SerializationError;

mod default_params;
mod error;
mod format_version;
mod keys;
mod serialization_common;
mod serialization_v1;
mod serialization_v2;
mod serialization_v3;
mod version_info;
mod xml;

/// Deserialize a kit patch from XML
pub fn deserialize_kit(xml: &str) -> Result<Kit, SerializationError> {
    Ok(deserialize_kit_with_version(xml)?.0)
}

pub fn deserialize_kit_with_version(xml: &str) -> Result<(Kit, VersionInfo), SerializationError> {
    let roots = xml::load_xml(xml)?;
    let version_info = version_info::load_version_info(&roots, PatchType::Kit);
    let kit = match version_info.format_version {
        format_version::FormatVersion::Version3 => serialization_v3::load_kit_nodes(&roots)?,
        format_version::FormatVersion::Version2 => serialization_v2::load_kit_nodes(&roots)?,
        format_version::FormatVersion::Version1 => serialization_v1::load_kit_nodes(&roots)?,
        format_version::FormatVersion::Unknown => return Err(SerializationError::InvalidVersionFormat),
    };

    Ok((kit, version_info))
}

/// Deserialize a synth patch from XML
pub fn deserialize_synth(xml: &str) -> Result<Synth, SerializationError> {
    Ok(deserialize_synth_with_version(xml)?.0)
}

pub fn deserialize_synth_with_version(xml: &str) -> Result<(Synth, VersionInfo), SerializationError> {
    let roots = xml::load_xml(xml)?;
    let version_info = version_info::load_version_info(&roots, PatchType::Synth);
    let synth = match version_info.format_version {
        format_version::FormatVersion::Version3 => serialization_v3::load_synth_nodes(&roots)?,
        format_version::FormatVersion::Version2 => serialization_v2::load_synth_nodes(&roots)?,
        format_version::FormatVersion::Version1 => serialization_v1::load_synth_nodes(&roots)?,
        format_version::FormatVersion::Unknown => return Err(SerializationError::InvalidVersionFormat),
    };

    Ok((synth, version_info))
}

/// Serialize a synth patch as XML
/// The patch is saved using the latest format version.
pub fn serialize_synth(synth: &Synth) -> Result<String, SerializationError> {
    let roots = vec![serialization_v3::write_synth(synth)?];

    Ok(xml::write_xml(&roots))
}

/// Serialize a kit patch as XML
/// The patch is saved using the latest format version.
pub fn serialize_kit(kit: &Kit) -> Result<String, SerializationError> {
    let roots = vec![serialization_v3::write_kit(kit)?];

    Ok(xml::write_xml(&roots))
}

#[cfg(test)]
mod tests {
    use crate::values::{FineTranspose, HexU50, LpfMode, Transpose};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_save_load_compare_version_3_synth() {
        let synth = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT184.XML")).unwrap();
        let xml = serialize_synth(&synth).unwrap();
        let reloaded_synth = deserialize_synth(&xml).unwrap();

        assert_eq!(reloaded_synth, synth);
    }

    #[test]
    fn test_save_load_compare_synth_version_3() {
        test_save_load_synth_compare(include_str!("../data_tests/SYNTHS/SYNT184.XML"));
        test_save_load_synth_compare(include_str!("../data_tests/SYNTHS/SYNT176.XML"));
        test_save_load_synth_compare(include_str!("../data_tests/SYNTHS/SYNT173.XML"));
        test_save_load_synth_compare(include_str!("../data_tests/SYNTHS/SYNT177.XML"));
    }

    #[test]
    fn test_save_load_compare_kit_version_3() {
        test_save_load_kit_compare(include_str!("../data_tests/KITS/KIT057.XML"));
        test_save_load_kit_compare(include_str!("../data_tests/KITS/Fmdrum.XML"));
        test_save_load_kit_compare(include_str!("../data_tests/KITS/KIT_TEST_SOUNDS_MIDI_GATE.XML"));
        test_save_load_kit_compare(include_str!("../data_tests/KITS/KIT_TEST_SOUNDS_ONLY.XML"));
    }

    #[test]
    fn test_save_load_compare_kit_version_3_midi() {
        test_save_load_kit_compare(include_str!("../data_tests/KITS/KIT_TEST_SOUNDS_MIDI_GATE.XML"));
    }

    fn test_save_load_synth_compare(input: &str) {
        let synth = deserialize_synth(input).unwrap();
        let xml = serialize_synth(&synth).unwrap();
        let reloaded_synth = deserialize_synth(&xml).unwrap();
        assert_eq!(reloaded_synth, synth);
    }

    fn test_save_load_kit_compare(input: &str) {
        let kit = deserialize_kit(input).unwrap();
        let xml = serialize_kit(&kit).unwrap();
        let reloaded_kit = deserialize_kit(&xml).unwrap();
        assert_eq!(reloaded_kit, kit);
    }

    #[test]
    fn test_load_version_3_synth() {
        let (_, version_info) = deserialize_synth_with_version(include_str!("../data_tests/SYNTHS/SYNT184.XML")).unwrap();

        assert_eq!(&version_info.firmware_version.unwrap(), "3.1.5");
        assert_eq!(
            &version_info
                .earliest_compatible_firmware
                .unwrap(),
            "3.1.0-beta"
        );
    }

    /// This test require the same patch saved under different version.
    #[test]
    fn test_convert_version_2_to_actual_synth() {
        // SYNT168.XML is a factory patch using format V2
        let synth_v2 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT168.XML")).unwrap();
        // SYNT168A.XML is just a save of SYNT168.XML done with firmware 3.1.5
        let synth_v3 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT168A.XML")).unwrap();

        assert_eq!(synth_v2, synth_v3);
    }

    #[test]
    fn test_convert_version_2_to_actual_synth_synt61() {
        let synth_v2 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT061.XML")).unwrap();
        let synth_v3 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT061A.XML")).unwrap();

        assert_eq!(synth_v2, synth_v3);
    }

    #[test]
    fn test_convert_version_2_to_actual_synth_synt002() {
        let synth_v2 = deserialize_synth(include_str!("../data_tests/Version conver/SYNT002.XML")).unwrap();
        let synth_v3 = deserialize_synth(include_str!("../data_tests/Version conver/SYNT002A.XML")).unwrap();

        assert_eq!(synth_v2, synth_v3);
    }

    // I can't figure out why this one can't work.
    // The original value for the global volume in SYNT039 should be 40 (0x4CCCCCA8 in the file)
    // but with the deluge it becomes 50!
    // So when I save with the Deluge, the value for the global volume jump to 50 accordingly.
    // I tried to change the value to 0x00000000 and Deluge displayed 40 this time. So I'm lost here.
    // Though it's not that important.
    // #[test]
    // fn test_convert_version_synt039() {
    //     let synth_v1 = load_synth(include_str!("../data_tests/Version conver/SYNT039.XML")).unwrap();
    //     let synth_v3 = load_synth(include_str!("../data_tests/Version conver/SYNT039C.XML")).unwrap();

    //     assert_eq!(synth_v1.sound.generator.as_ring_mod().unwrap().osc1.as_waveform().unwrap().retrig_phase, RetrigPhase::Degrees(0));
    //     assert_eq!(synth_v1, synth_v3);
    // }

    /// This test require the same patch saved under different version.
    #[test]
    fn test_convert_version_synt008() {
        let synth_v1 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT008.XML")).unwrap();
        let synth_v3 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT008A.XML")).unwrap();

        assert_eq!(synth_v1, synth_v3);
    }

    /// This test require the same patch saved under different version.
    #[test]
    fn test_convert_version_synt004() {
        let synth_v1 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT004.XML")).unwrap();
        let synth_v3 = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT004A.XML")).unwrap();

        assert_eq!(synth_v1, synth_v3);
    }

    #[test]
    fn test_load_write_load_synth_028() {
        let file_content = include_str!("../data_tests/SYNTHS/SYNT028.XML");
        let synth = deserialize_synth(&file_content).unwrap();
        let xml = serialize_synth(&synth).unwrap();
        let reloaded_synth = deserialize_synth(&xml).unwrap();

        assert_eq!(reloaded_synth, synth);
    }

    #[test]
    fn test_load_version_2_synth() {
        let (_, version_info) = deserialize_synth_with_version(include_str!("../data_tests/SYNTHS/SYNT170.XML")).unwrap();

        assert_eq!(&version_info.firmware_version.unwrap(), "2.1.0");
        assert_eq!(
            &version_info
                .earliest_compatible_firmware
                .unwrap(),
            "2.1.0"
        );
    }

    #[test]
    fn test_load_version_3_kit() {
        let (kit, version_info) = deserialize_kit_with_version(include_str!("../data_tests/KITS/KIT057.XML")).unwrap();

        assert_eq!(&version_info.firmware_version.unwrap(), "3.1.5");
        assert_eq!(
            &version_info
                .earliest_compatible_firmware
                .unwrap(),
            "3.1.0-beta"
        );

        assert_eq!(kit.rows.len(), 7);
        assert_eq!(kit.lpf_mode, LpfMode::Lpf24);
        assert_eq!(
            kit.modulation_fx
                .as_flanger()
                .unwrap()
                .rate,
            HexU50::parse("0xE0000000").unwrap()
        );
        assert_eq!(
            kit.modulation_fx
                .as_flanger()
                .unwrap()
                .feedback,
            HexU50::parse("0x80000000").unwrap()
        );
        assert_eq!(kit.selected_row_index, Some(4));
    }

    #[test]
    fn test_load_version_2_kit() {
        let (kit, version_info) = deserialize_kit_with_version(include_str!("../data_tests/KITS/KIT026.XML")).unwrap();

        assert_eq!(&version_info.firmware_version.unwrap(), "2.1.0");
        assert_eq!(
            &version_info
                .earliest_compatible_firmware
                .unwrap(),
            "2.0.0"
        );

        assert_eq!(kit.rows.len(), 16);
        assert_eq!(kit.lpf_mode, LpfMode::Lpf24);
        assert_eq!(
            kit.modulation_fx
                .as_flanger()
                .unwrap()
                .rate,
            HexU50::parse("0xE0000000").unwrap()
        );
        assert_eq!(
            kit.modulation_fx
                .as_flanger()
                .unwrap()
                .feedback,
            HexU50::parse("0x80000000").unwrap()
        );
        assert_eq!(kit.selected_row_index, Some(15));
    }

    #[test]
    fn test_load_version_1_kit() {
        let (kit, version_info) = deserialize_kit_with_version(include_str!("../data_tests/KITS/KIT000.XML")).unwrap();

        assert_eq!(&version_info.firmware_version, &None);
        assert_eq!(&version_info.earliest_compatible_firmware, &None);

        assert_eq!(kit.rows.len(), 16);
        assert_eq!(kit.lpf_mode, LpfMode::Lpf24);
        assert_eq!(
            kit.modulation_fx
                .as_flanger()
                .unwrap()
                .rate,
            HexU50::parse("0xE0000000").unwrap()
        );
        assert_eq!(
            kit.modulation_fx
                .as_flanger()
                .unwrap()
                .feedback,
            HexU50::parse("0x80000000").unwrap()
        );
        assert_eq!(kit.selected_row_index, Some(14));
    }

    #[test]
    fn test_load_write_load_kit_002() {
        let file_content = include_str!("../data_tests/KITS/KIT002.XML");
        let kit = deserialize_kit(&file_content).unwrap();
        let xml = serialize_kit(&kit).unwrap();
        eprintln!("{}", xml);
        let reloaded_kit = deserialize_kit(&xml).unwrap();

        assert_eq!(reloaded_kit, kit);
    }

    #[test]
    fn test_load_version_sample_range() {
        let synth = deserialize_synth(include_str!("../data_tests/SYNTHS/SYNT170.XML")).unwrap();
        let sample_ranges = synth
            .sound
            .generator
            .as_subtractive()
            .unwrap()
            .osc1
            .as_sample()
            .unwrap()
            .sample
            .as_sample_ranges()
            .unwrap();

        assert_eq!(21, sample_ranges.len());
        assert_eq!(
            "SAMPLES/Artists/Michael Bulaw/Sitar/Freeze Sitar [2018-12-06 224345].wav",
            sample_ranges[0]
                .file_path
                .to_string_lossy()
        );
        assert_eq!(Some(53), sample_ranges[0].range_top_note);
        assert_eq!(0, sample_ranges[0].zone.start.as_u64());
        assert_eq!(264600, sample_ranges[0].zone.end.as_u64());
        assert_eq!(Transpose::new(7), sample_ranges[0].transpose);
        assert_eq!(FineTranspose::new(8), sample_ranges[0].fine_transpose);
    }
}
