use xmltree::Element;

use super::{
    format_version::{detect_format_version, FormatVersion},
    keys, xml,
};

#[derive(PartialEq, Debug)]
pub struct VersionInfo {
    pub firmware_version: Option<String>,
    pub earliest_compatible_firmware: Option<String>,
    pub format_version: FormatVersion,
}

pub fn load_version_info(roots: &[Element], element_type: &str) -> VersionInfo {
    // Yeah it's not the best possible because I'm reading the same information twice.
    // Also it's easier for testing to have `detect_format_version` independent.
    VersionInfo {
        firmware_version: load_version(roots, element_type, keys::FIRMWARE_VERSION),
        earliest_compatible_firmware: load_version(roots, element_type, keys::EARLIEST_COMPATIBLE_FIRMWARE),
        format_version: detect_format_version(roots, element_type).unwrap_or(FormatVersion::Unknown),
    }
}

fn load_version(roots: &[Element], element_type: &str, key: &str) -> Option<String> {
    if let Some(version) = xml::get_opt_element(roots, key).map(xml::get_text) {
        return Some(version);
    }

    if let Some(node) = xml::get_opt_element(roots, element_type) {
        if let Some(version) = xml::get_opt_attribute(node, key).cloned() {
            return Some(version);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_version_sound() {
        assert_eq!(
            VersionInfo {
                firmware_version: Some("3.1.5".to_string()),
                earliest_compatible_firmware: Some("3.1.0-beta".to_string()),
                format_version: FormatVersion::Version3,
            },
            load_version_info(
                &xml::load_xml(include_str!("../data_tests/SYNTHS/SYNT184.XML")).unwrap(),
                keys::SOUND
            )
        );
    }

    #[test]
    fn test_detect_format_version_kit() {
        assert_eq!(
            VersionInfo {
                firmware_version: Some("3.1.5".to_string()),
                earliest_compatible_firmware: Some("3.1.0-beta".to_string()),
                format_version: FormatVersion::Version3,
            },
            load_version_info(
                &xml::load_xml(include_str!("../data_tests/KITS/KIT057.XML")).unwrap(),
                keys::KIT
            )
        );

        assert_eq!(
            VersionInfo {
                firmware_version: Some("2.1.0".to_string()),
                earliest_compatible_firmware: Some("2.0.0".to_string()),
                format_version: FormatVersion::Version2,
            },
            load_version_info(
                &xml::load_xml(include_str!("../data_tests/KITS/KIT026.XML")).unwrap(),
                keys::KIT
            )
        );

        assert_eq!(
            VersionInfo {
                firmware_version: None,
                earliest_compatible_firmware: None,
                format_version: FormatVersion::Version1,
            },
            load_version_info(
                &xml::load_xml(include_str!("../data_tests/KITS/KIT000.XML")).unwrap(),
                keys::KIT
            )
        );
    }
}
