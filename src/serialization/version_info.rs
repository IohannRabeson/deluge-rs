use xmltree::Element;

use super::{keys, patch_type::PatchType, xml};

#[derive(PartialEq, Debug)]
pub struct VersionInfo {
    pub firmware_version: Option<String>,
    pub earliest_compatible_firmware: Option<String>,
    pub format_version: FormatVersion,
}

pub fn load_version_info(roots: &[Element], patch_type: PatchType) -> VersionInfo {
    let earliest_compatible_firmware = load_version(roots, patch_type, keys::EARLIEST_COMPATIBLE_FIRMWARE);

    VersionInfo {
        firmware_version: load_version(roots, patch_type, keys::FIRMWARE_VERSION),
        earliest_compatible_firmware: earliest_compatible_firmware.clone(),
        format_version: earliest_compatible_firmware.into(),
    }
}

fn load_version(roots: &[Element], patch_type: PatchType, key: &str) -> Option<String> {
    if let Some(version) = xml::get_opt_element(roots, key).map(xml::get_text) {
        return Some(version);
    }

    if let Some(node) = xml::get_opt_element(roots, patch_type.get_key()) {
        if let Some(version) = xml::get_opt_attribute(node, key).cloned() {
            return Some(version);
        }
    }

    None
}

/// Deluge format version
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormatVersion {
    /// No version specified
    None,
    /// A version has been parsed but it's not supported
    Unsupported,
    /// The initial version of the Deluge format. Nothing was specified actually.
    Version1,
    /// This version introduces the firmwareVersion information in the data stored as content of root node.
    Version2,
    /// This version uses more attributes instead of children.
    Version3,
}

fn parse_version(version_string: String) -> FormatVersion {
    if let Some(version) = version_compare::Version::from(&version_string) {
        if let Some(major) = version.parts().first() {
            return match major.to_string().as_str() {
                "1" => FormatVersion::Version1,
                "2" => FormatVersion::Version2,
                "3" => FormatVersion::Version3,
                _ => FormatVersion::Unsupported,
            };
        }
    }

    FormatVersion::None
}

impl From<Option<String>> for FormatVersion {
    fn from(version: Option<String>) -> Self {
        match version {
            Some(version_string) => parse_version(version_string),
            None => FormatVersion::Version1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

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
                PatchType::Synth
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
                PatchType::Kit
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
                PatchType::Kit
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
                PatchType::Kit
            )
        );
    }

    #[test_case("1", FormatVersion::Version1)]
    #[test_case("2", FormatVersion::Version2)]
    #[test_case("3", FormatVersion::Version3)]
    #[test_case("3.0.0", FormatVersion::Version3)]
    #[test_case("3.0.0-beta", FormatVersion::Version3)]
    #[test_case("666", FormatVersion::Unsupported)]
    #[test_case("0", FormatVersion::Unsupported)]
    #[test_case("HEU!", FormatVersion::None)]
    fn test_parse_version(input: &str, expected: FormatVersion) {
        assert_eq!(parse_version(input.to_string()), expected);
    }
}
