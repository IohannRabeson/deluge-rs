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

impl From<Option<String>> for FormatVersion {
    fn from(version: Option<String>) -> Self {
        match version {
            Some(version_string) => parse_version(version_string),
            None => FormatVersion::Version1,
        }
    }
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

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::serialization::format_version::parse_version;

    use super::FormatVersion;

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
