use std::borrow::Cow;

use crate::Error;

use xmltree::Element;

use super::{keys, xml};

/// Deluge format version
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormatVersion {
    // The initial version of the format
    Version1,
    // This version introduces the firmwareVersion information in the data
    Version2,
    // This version uses more attributes instead of children
    Version3,
}

type VersionFunctionDetection = fn(roots: &[Element], element_type: &str) -> bool;

/// This version, there is no firmwareVersion element, only a kit element.
fn is_version_1(roots: &[Element], element_type: &str) -> bool {
    if xml::get_element(roots, keys::FIRMWARE_VERSION).is_err() && xml::get_element(roots, element_type).is_ok() {
        return true;
    }

    if let Ok(firmware_version_node) = xml::get_element(roots, keys::FIRMWARE_VERSION) {
        if let Some(firmware_version) = firmware_version_node.get_text() {
            return check_for_version(&firmware_version, '1');
        }
    }

    false
}

/// This version, there is one firmwareVersion element and a kit element
fn is_version_2(roots: &[Element], element_type: &str) -> bool {
    if xml::get_element(roots, element_type).is_err() {
        return false;
    }

    if let Ok(firmware_version_node) = xml::get_element(roots, keys::FIRMWARE_VERSION) {
        if let Some(firmware_version) = firmware_version_node.get_text() {
            return check_for_version(&firmware_version, '2');
        }
    }

    false
}

/// This version, a kit element with an attribute firmwareVersion.
/// It seems attributes are used almost everywhere now.
fn is_version_3(roots: &[Element], element_type: &str) -> bool {
    if let Ok(kit_node) = xml::get_element(roots, element_type) {
        if let Ok(firmware_version) = xml::get_attribute(kit_node, keys::FIRMWARE_VERSION) {
            return check_for_version(&Cow::Borrowed(firmware_version), '3');
        }
    }

    false
}

fn check_for_version(text: &str, expected_first_char: char) -> bool {
    match text.chars().next() {
        Some(first_char) => first_char == expected_first_char,
        None => false,
    }
}

pub fn detect_kit_format_version(roots: &[Element]) -> Result<FormatVersion, Error> {
    detect_format_version(roots, keys::KIT)
}

pub fn detect_sound_format_version(roots: &[Element]) -> Result<FormatVersion, Error> {
    detect_format_version(roots, keys::SOUND)
}

fn detect_format_version(roots: &[Element], element_type: &str) -> Result<FormatVersion, Error> {
    // Notice we check the newest versions first, but this is because version 1 does not contains any version infos.
    let functions: Vec<(VersionFunctionDetection, FormatVersion)> = vec![
        (is_version_3, FormatVersion::Version3),
        (is_version_2, FormatVersion::Version2),
        (is_version_1, FormatVersion::Version1),
    ];

    for f in &functions {
        if f.0(roots, element_type) {
            return Ok(f.1);
        }
    }

    Err(Error::InvalidVersionFormat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_version_sound() {
        assert_eq!(
            FormatVersion::Version3,
            detect_sound_format_version(&xml::load_xml(include_str!("../data_tests/SYNTHS/SYNT184.XML")).unwrap()).unwrap()
        );
        assert_eq!(
            FormatVersion::Version2,
            detect_sound_format_version(&xml::load_xml(include_str!("../data_tests/SYNTHS/SYNT020.XML")).unwrap()).unwrap()
        );
        assert_eq!(
            FormatVersion::Version1,
            detect_sound_format_version(&xml::load_xml(include_str!("../data_tests/SYNTHS/SYNT000.XML")).unwrap()).unwrap()
        );
    }

    #[test]
    fn test_detect_format_version_kit() {
        assert_eq!(
            FormatVersion::Version3,
            detect_kit_format_version(&xml::load_xml(include_str!("../data_tests/KITS/KIT057.XML")).unwrap()).unwrap()
        );
        assert_eq!(
            FormatVersion::Version2,
            detect_kit_format_version(&xml::load_xml(include_str!("../data_tests/KITS/KIT026.XML")).unwrap()).unwrap()
        );
        assert_eq!(
            FormatVersion::Version1,
            detect_kit_format_version(&xml::load_xml(include_str!("../data_tests/KITS/KIT000.XML")).unwrap()).unwrap()
        );
    }
}
