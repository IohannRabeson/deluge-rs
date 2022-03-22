use crate::{Error, Kit, Synth};

mod default_params;
mod keys;
mod serialization_common;
mod serialization_v1;
mod serialization_v2;
mod serialization_v3;
mod version_detection;
mod xml;

/// Load a kit patch from XML
pub fn load_kit(xml: &str) -> Result<Kit, Error> {
    let roots = xml::load_xml(xml)?;
    let version = version_detection::detect_kit_format_version(&roots)?;

    match version {
        version_detection::FormatVersion::Version3 => serialization_v3::load_kit_nodes(&roots),
        version_detection::FormatVersion::Version2 => serialization_v2::load_kit_nodes(&roots),
        version_detection::FormatVersion::Version1 => serialization_v1::load_kit_nodes(&roots),
    }
}

/// Load a synth patch from XML
pub fn load_synth(xml: &str) -> Result<Synth, Error> {
    let roots = xml::load_xml(xml)?;
    let version = version_detection::detect_synth_format_version(&roots)?;

    match version {
        version_detection::FormatVersion::Version3 => serialization_v3::load_synth_nodes(&roots),
        version_detection::FormatVersion::Version2 => serialization_v2::load_synth_nodes(&roots),
        version_detection::FormatVersion::Version1 => serialization_v1::load_synth_nodes(&roots),
    }
}

/// Save a synth patch as XML
/// The patch is saved using the latest format version.
pub fn save_synth(synth: &Synth) -> Result<String, Error> {
    let roots = vec![serialization_v3::write_synth(synth)?];

    Ok(xml::write_xml(&roots))
}

/// Save a kit patch as XML
/// The patch is saved using the latest format version.
pub fn save_kit(kit: &Kit) -> Result<String, Error> {
    let roots = vec![serialization_v3::write_kit(kit)?];

    Ok(xml::write_xml(&roots))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_save_load_compare_version_3_synth() {
        let synth = load_synth(include_str!("../data_tests/SYNTHS/SYNT184.XML")).unwrap();
        let xml = save_synth(&synth).unwrap();
        let reloaded_synth = load_synth(&xml).unwrap();

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
        let synth = load_synth(input).unwrap();
        let xml = save_synth(&synth).unwrap();
        let reloaded_synth = load_synth(&xml).unwrap();
        assert_eq!(reloaded_synth, synth);
    }

    fn test_save_load_kit_compare(input: &str) {
        let kit = load_kit(input).unwrap();
        let xml = save_kit(&kit).unwrap();
        let reloaded_kit = load_kit(&xml).unwrap();
        assert_eq!(reloaded_kit, kit);
    }

    #[test]
    fn test_load_version_3_synth() {
        let synth = load_synth(include_str!("../data_tests/SYNTHS/SYNT184.XML")).unwrap();

        assert_eq!(&synth.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&synth.earliest_compatible_firmware.unwrap(), "3.1.0-beta");
    }

    #[test]
    fn test_load_version_3_kit() {
        let kit = load_kit(include_str!("../data_tests/KITS/KIT057.XML")).unwrap();

        assert_eq!(&kit.firmware_version.unwrap(), "3.1.5");
        assert_eq!(&kit.earliest_compatible_firmware.unwrap(), "3.1.0-beta");
        assert_eq!(kit.rows.len(), 7);
    }

    /// This test require the same patch saved under different version.
    #[test]
    fn test_convert_version_2_to_actual_synth() {
        // SYNT168.XML is a factory patch using format V2
        let synth_v2 = load_synth(include_str!("../data_tests/SYNTHS/SYNT168.XML")).unwrap();
        // SYNT168A.XML is just a save of SYNT168.XML done with firmware 3.1.5
        let synth_v3 = load_synth(include_str!("../data_tests/SYNTHS/SYNT168A.XML")).unwrap();

        assert_eq!(synth_v2, synth_v3);
    }

    /// This test require the same patch saved under different version.
    #[test]
    fn test_convert_version_2_to_actual_synt008() {
        let synth_v2 = load_synth(include_str!("../data_tests/SYNTHS/SYNT008.XML")).unwrap();
        let synth_v3 = load_synth(include_str!("../data_tests/SYNTHS/SYNT008A.XML")).unwrap();

        assert_eq!(synth_v2, synth_v3);
    }

    #[test]
    fn test_load_write_load_synth_028() {
        let file_content = include_str!("../data_tests/SYNTHS/SYNT028.XML");
        let synth = load_synth(&file_content).unwrap();
        let xml = save_synth(&synth).unwrap();
        let reloaded_synth = load_synth(&xml).unwrap();

        assert_eq!(reloaded_synth, synth);
    }

    #[test]
    fn test_load_version_2_synth() {
        let kit = load_synth(include_str!("../data_tests/SYNTHS/SYNT170.XML")).unwrap();

        assert_eq!(&kit.firmware_version.unwrap(), "2.1.0");
        assert_eq!(&kit.earliest_compatible_firmware.unwrap(), "2.1.0");
    }

    #[test]
    fn test_load_version_2_kit() {
        let kit = load_kit(include_str!("../data_tests/KITS/KIT026.XML")).unwrap();

        assert_eq!(&kit.firmware_version.unwrap(), "2.1.0");
        assert_eq!(&kit.earliest_compatible_firmware.unwrap(), "2.0.0");
        assert_eq!(kit.rows.len(), 16);
    }

    #[test]
    fn test_load_version_1_kit() {
        let kit = load_kit(include_str!("../data_tests/KITS/KIT000.XML")).unwrap();

        assert_eq!(&kit.firmware_version, &None);
        assert_eq!(&kit.earliest_compatible_firmware, &None);
        assert_eq!(kit.rows.len(), 16);
    }

    #[test]
    fn test_load_write_load_kit_002() {
        let file_content = include_str!("../data_tests/KITS/KIT002.XML");
        let kit = load_kit(&file_content).unwrap();
        let xml = save_kit(&kit).unwrap();
        eprintln!("{}", xml);
        let reloaded_kit = load_kit(&xml).unwrap();

        assert_eq!(reloaded_kit, kit);
    }
}
