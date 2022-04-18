use std::str::FromStr;

use nom::Finish;

use crate::PatchType;

pub type ParseError = nom::error::Error<String>;

/// A parsed patch name
///
/// There are 2 types of patch name, standard and custom.
/// 
/// The standard patch name looks like "KIT000", "KIT001", "KIT001A". It contains 3 parts, a tag which is KIT or SYNT then follows
/// the number and finally the optional suffix. I assume the suffix is always one char but I'm not sure about that. Also I'm not
/// sure what happens on the deluge when you try to save more than 26 variations.
/// I guess it says CANT but I never tried it -- good way of doing it is to create using the computer
/// artificial variation file then try to create the last one on the Deluge.
/// 
/// The custom patch name is simpler, it's any chars with at the end an optional number preceded by a space.
/// Example: "HELLO WORLD", "HELLO WORLD 12".
/// ```
/// use deluge::{PatchType, PatchName};
/// use std::str::FromStr;
/// let patch_name = PatchName::from_str("SYNT234R").unwrap();
/// assert_eq!(
///     patch_name,
///     PatchName::Standard{ patch_type: PatchType::Synth, number: 234, suffix: Some('R') },
/// )
/// ```
/// Todo: the field number should be an integer limited to [0; 999] (like deluge::values::int8 but for u16).
#[derive(Debug, PartialEq, Eq)]
pub enum PatchName {
    Standard {
        patch_type: PatchType,
        number: u16,
        suffix: Option<char>,
    },
    Custom {
        name: String,
        number: Option<u16>,
    },
}

impl PatchName {
    fn standard_to_string(patch_type: PatchType, number: u16, suffix: Option<char>) -> String {
        let mut buffer = String::with_capacity(7);

        buffer.push_str(patch_type.get_standard_patch_base_name());
        buffer.push_str(&format!("{:03}", number));

        if let Some(suffix) = suffix {
            buffer.push(suffix);
        }

        buffer
    }

    fn custom_to_string(name: &str, number: Option<u16>) -> String {
        let mut buffer = String::new();

        buffer.push_str(name);
        if let Some(number) = number {
            buffer.push_str(&format!(" {}", number));
        }

        buffer
    }
}

impl FromStr for PatchName {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match parser::parse_patch_name(input).finish() {
            Ok(patch_name) => Ok(patch_name.1),
            Err(nom::error::Error { input, code }) => Err(nom::error::Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

impl ToString for PatchName {
    fn to_string(&self) -> String {
        match self {
            PatchName::Standard {
                patch_type,
                number,
                suffix,
            } => Self::standard_to_string(*patch_type, *number, *suffix),
            PatchName::Custom { name, number } => Self::custom_to_string(name, *number),
        }
    }
}

mod parser {
    use std::num::ParseIntError;

    use super::*;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{digit1, one_of},
        combinator::{map, map_res, opt, value},
        sequence::tuple,
        IResult,
    };

    const MAX_PATCH_NAME_NUMBER: u16 = 999;

    #[derive(PartialEq, Eq, Clone, Debug, thiserror::Error)]
    enum ParseDigitError {
        #[error("failed to parse integer: too many digits (max is 3)")]
        TooManyDigits,
        #[error("failed to parse integer: value too big (max is 999)")]
        Overflow,
        #[error("failed to parse integer: {0}")]
        InvalidInteger(#[from] ParseIntError),
    }

    fn map_number_3_digits(input: &str) -> Result<u16, ParseDigitError> {
        if input.len() > 3 {
            return Err(ParseDigitError::TooManyDigits);
        }

        let number = u16::from_str(input)?;

        if number > MAX_PATCH_NAME_NUMBER {
            return Err(ParseDigitError::Overflow);
        }

        Ok(number)
    }

    fn parse_3digits(input: &str) -> IResult<&str, u16> {
        map_res(digit1, map_number_3_digits)(input)
    }

    const BASE_NAME_KIT: &str = "KIT";
    const BASE_NAME_SYNTH: &str = "SYNT";

    fn parse_patch_type(input: &str) -> IResult<&str, PatchType> {
        alt((
            value(PatchType::Kit, tag(BASE_NAME_KIT)),
            value(PatchType::Synth, tag(BASE_NAME_SYNTH)),
        ))(input)
    }

    fn parse_suffix(input: &str) -> IResult<&str, char> {
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(input)
    }

    fn parse_standard_patch_name(input: &str) -> IResult<&str, PatchName> {
        let parser = tuple((parse_patch_type, parse_3digits, opt(parse_suffix)));

        map(parser, |(patch_type, number, suffix)| PatchName::Standard {
            patch_type,
            number,
            suffix,
        })(input)
    }

    fn parse_custom_patch_name(input: &str) -> IResult<&str, PatchName> {
        match input.rfind(' ') {
            Some(index) => {
                let potential_number = &input[index + 1..];

                match potential_number.parse::<u16>() {
                    Ok(number) => {
                        let name = &input[0..index];

                        Ok((
                            "",
                            PatchName::Custom {
                                name: name.to_string(),
                                number: Some(number),
                            },
                        ))
                    }
                    Err(_) => Ok((
                        "",
                        PatchName::Custom {
                            name: input.to_string(),
                            number: None,
                        },
                    )),
                }
            }
            None => Ok((
                "",
                PatchName::Custom {
                    name: input.to_string(),
                    number: None,
                },
            )),
        }
    }

    /// Parse any patch name properly formatted.
    /// This is the entry point of this module.
    pub(crate) fn parse_patch_name(input: &str) -> IResult<&str, PatchName> {
        alt((parse_standard_patch_name, parse_custom_patch_name))(input)
    }

    #[cfg(test)]
    mod tests {
        use crate::PatchType;
        use test_case::test_case;

        use super::*;

        #[test_case("KIT", PatchType::Kit ; "KIT")]
        #[test_case("SYNT", PatchType::Synth ; "SYNTH")]
        fn test_parse_patch_type_success(input: &str, expected_result: PatchType) {
            let (_remaining, result) = parse_patch_type(input).unwrap();

            assert_eq!(expected_result, result)
        }

        #[test_case("KYT" ; "KYT")]
        #[test_case("SINT" ; "SINT")]
        fn test_parse_patch_type_fail(input: &str) {
            assert!(parse_patch_type(input).is_err());
        }

        #[test_case("KIT000", PatchType::Kit, 0, None ; "KIT000")]
        #[test_case("SYNT000", PatchType::Synth, 0, None ; "SYNT000")]
        #[test_case("SYNT123A", PatchType::Synth, 123, Some('A') ; "SYNT123A")]
        fn test_parse_standard_patch_name_success(
            input: &str,
            expected_patch_type: PatchType,
            expected_number: u16,
            expected_suffix: Option<char>,
        ) {
            let (_, result) = parse_standard_patch_name(input).unwrap();
            let expected_result = PatchName::Standard {
                patch_type: expected_patch_type,
                number: expected_number,
                suffix: expected_suffix,
            };

            assert_eq!(expected_result, result)
        }

        #[test_case("KITI000" ; "KITI000")]
        #[test_case("SYNTO000" ; "SYNTO000")]
        #[test_case("SYN1T123A" ; "SYN1T123A")]
        #[test_case("KIT0000" ; "KIT0000")]
        #[test_case("KIT1000" ; "KIT1000")]
        fn test_parse_standard_patch_name_fail(input: &str) {
            assert!(parse_standard_patch_name(input).is_err());
        }

        #[test_case("KITO", "KITO", None ; "KITO")]
        #[test_case("SYNT 1", "SYNT", Some(1) ; "SYNT 1")]
        #[test_case("KIT 123", "KIT", Some(123) ; "KIT 123")]
        #[test_case("KIT", "KIT", None ; "KIT")]
        #[test_case("SYNT", "SYNT", None ; "SYNT")]
        fn test_parse_custom_patch_name_success(input: &str, expected_name: &str, expected_number: Option<u16>) {
            let (_, result) = parse_custom_patch_name(input).unwrap();
            let expected_result = PatchName::Custom {
                name: expected_name.to_string(),
                number: expected_number,
            };

            assert_eq!(expected_result, result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("KIT000", PatchType::Kit, 0, None ; "KIT000")]
    #[test_case("KIT000A", PatchType::Kit, 0, Some('A') ; "KIT000A")]
    #[test_case("SYNT123V", PatchType::Synth, 123, Some('V') ; "SYNT123V")]
    fn parse_valid_input_standard_test(
        input: &str,
        expected_patch_type: PatchType,
        expected_number: u16,
        expected_suffix: Option<char>,
    ) {
        let expected = PatchName::Standard {
            patch_type: expected_patch_type,
            number: expected_number,
            suffix: expected_suffix,
        };

        assert_eq!(expected, PatchName::from_str(input).unwrap());
    }

    #[test_case("KIT", "KIT", None ; "KIT")]
    #[test_case("KIKI FLORIDA0101", "KIKI FLORIDA0101", None ; "standard without number")]
    #[test_case("YO 123", "YO", Some(123) ; "standard with number")]
    fn parse_valid_input_custom_test(input: &str, expected_name: &str, expected_number: Option<u16>) {
        let expected = PatchName::Custom {
            name: expected_name.to_string(),
            number: expected_number,
        };

        assert_eq!(expected, PatchName::from_str(input).unwrap());
    }
}
