use std::str::FromStr;

use nom::Finish;

use crate::PatchType;

pub type ParseError = nom::error::Error<String>;

/// A parsed patch name
/// 
/// There are 2 type of patch name:
///  - Standard: "SYNT|KIT<3DIGITS>[<SUFFIX>]"
///  - Custom: "ALPHANUMSPACE1 ALPHANUM[ NUMBER]"
/// ```
/// use deluge::{PatchType, PatchName};
/// use std::str::FromStr;
/// let patch_name = PatchName::from_str("SYNT234R").unwrap();
/// assert_eq!(
///     patch_name,
///     PatchName::Standard{ patch_type: PatchType::Synth, number: 234, suffix: Some('R') },
/// )
/// ```
// #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
// pub struct PatchName {
//     pub name: String,
//     pub number: Option<u16>,
//     pub suffix: Option<String>,
// }

#[derive(Debug, PartialEq, Eq)]
pub enum PatchName {
    Standard{ patch_type: PatchType, number: u16, suffix: Option<char> },
    CustomName{ name: String, number: Option<u16> },
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
            PatchName::Standard { patch_type, number, suffix } => {
                Self::standard_to_string(*patch_type, *number, *suffix)
            },
            PatchName::CustomName { name, number } => {
                Self::custom_to_string(name, *number)
            }
        }        
    }
}

mod parser {
    use super::*;

    use nom::{
        character::{complete::{digit1, one_of, alphanumeric1, space0}},
        combinator::{map, opt, map_res, value}, IResult, branch::alt, bytes::complete::tag, sequence::{tuple, preceded, pair}, multi::many1,
    };
    
    /// TODO: parse only 3 digits!
    fn parse_3digits(input: &str) -> IResult<&str, u16> {
        map_res(digit1, u16::from_str)(input)
    }

    const BASE_NAME_KIT: &str = "KIT";
    const BASE_NAME_SYNTH: &str = "SYNT";

    // fn parse_patch_type_kit(input: &str) -> IResult<&str, PatchType> {
    //     map(tag(BASE_NAME_KIT), |parsed_tag| { 
    //         debug_assert!(parsed_tag == BASE_NAME_KIT);
    //         PatchType::Kit 
    //     })(input)
    // }

    // fn parse_patch_type_synth(input: &str) -> IResult<&str, PatchType> {
    //     map(tag(BASE_NAME_SYNTH), |parsed_tag| {
    //         debug_assert!(parsed_tag == BASE_NAME_SYNTH);
    //         PatchType::Kit 
    //     })(input)
    // }

    fn parse_patch_type(input: &str) -> IResult<&str, PatchType> {
        alt(
            (value(PatchType::Kit, tag(BASE_NAME_KIT)), value(PatchType::Synth, tag(BASE_NAME_SYNTH)))
        ) (input)
    }

    fn parse_suffix(input: &str) -> IResult<&str, char> {
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(input)
    }

    fn parse_standard_patch_name(input: &str) -> IResult<&str, PatchName> {
        let parser = tuple((parse_patch_type, parse_3digits, opt(parse_suffix)));

        map(parser, |(patch_type, number, suffix)| PatchName::Standard { patch_type, number, suffix })(input)
    }

    // fn parse_custom_name(input: &str) -> IResult<&str, String> {
    //     let parser_start = many1(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"));
    //     let parser_body = many1(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 "));
    //     let parser_end = many1(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"));
    //     let mut parser = tuple(parser_start, parser_body, parser_end);

    //     map(parser, |(start, body)|format!("{}{}", start.into_iter().collect::<String>(), body.into_iter().collect::<String>())) (input)
    // }

    // fn parse_custom_number(input: &str) -> IResult<&str, u16> {
    //     let mut parser = preceded(nom::character::complete::char(' '), map_res(digit1, str::parse));

    //     parser(input)
    // }

    fn parse_custom_patch_name(input: &str) -> IResult<&str, PatchName> {
        match input.rfind(' ') {
            Some(index) => {
                let potential_number = &input[index + 1..];

                match potential_number.parse::<u16>() {
                    Ok(number) => {
                        let name = &input[0..index];

                        Ok(("", PatchName::CustomName { name: name.to_string(), number: Some(number) }))
                    },
                    Err(_) => {
                        Ok(("", PatchName::CustomName { name: input.to_string(), number: None }))
                    },
                }
            },
            None => {
                Ok(("", PatchName::CustomName { name: input.to_string(), number: None }))
            }
        }
    }

    pub(crate) fn parse_patch_name(input: &str) -> IResult<&str, PatchName> {
        alt((parse_standard_patch_name, parse_custom_patch_name))(input)
    }

    #[cfg(test)]
    mod tests {
        use test_case::test_case;
        use crate::PatchType;

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
            assert!( parse_patch_type(input).is_err() );
        }

        #[test_case("KIT000", PatchType::Kit, 0, None ; "KIT000")]
        #[test_case("SYNT000", PatchType::Synth, 0, None ; "SYNT000")]
        #[test_case("SYNT123A", PatchType::Synth, 123, Some('A') ; "SYNT123A")]
        fn test_parse_standard_patch_name_success(input: &str, expected_patch_type: PatchType, expected_number: u16, expected_suffix: Option<char>) {
            let (_, result) = parse_standard_patch_name(input).unwrap();
            let expected_result = PatchName::Standard { patch_type: expected_patch_type, number: expected_number, suffix: expected_suffix };
            
            assert_eq!(expected_result, result)
        }

        #[test_case("KITI000" ; "KITI000")]
        #[test_case("SYNTO000" ; "SYNTO000")]
        #[test_case("SYN1T123A" ; "SYN1T123A")]
        fn test_parse_standard_patch_name_fail(input: &str) {
            assert!( parse_standard_patch_name(input).is_err() );
        }

        #[test_case("KITO", "KITO", None ; "KITO")]
        #[test_case("SYNT 1", "SYNT", Some(1) ; "SYNT 1")]
        #[test_case("KIT 123", "KIT", Some(123) ; "KIT 123")]
        #[test_case("KIT", "KIT", None ; "KIT")]
        #[test_case("SYNT", "SYNT", None ; "SYNT")]
        fn test_parse_custom_patch_name_success(input: &str, expected_name: &str, expected_number: Option<u16>) {
            let (_, result) = parse_custom_patch_name(input).unwrap();
            let expected_result = PatchName::CustomName { name: expected_name.to_string(), number: expected_number };
            
            assert_eq!(expected_result, result)
        }


        
        // #[test_case("YO", "YO" ; "YO")]
        // #[test_case("SYNT A", "SYNT A" ; "SYNT A")]
        // #[test_case("SY12NT A", "SY12NT A" ; "SY12NT A")]
        // #[test_case("SY12NT A121", "SY12NT A121" ; "SY12NT A121")]
        // #[test_case("123", "123" ; "123")]
        // #[test_case("123YO", "123YO" ; "123YO")]
        // fn test_parse_custom_name_success(input: &str, expected_name: &str) {
        //     let (_, result) = parse_custom_name(input).unwrap();

        //     assert_eq!(expected_name, result);
        // }

        // #[test_case(" YO" ; " YO")]
        // #[test_case(" " ; "SPACE")]
        // fn test_parse_custom_name_fail(input: &str) {
        //     assert!(parse_custom_name(input).is_err());
        // }
    }
}

// fn parse_alpha(input: &str) -> IResult<&str, String> {
//     map_res(alpha1, str::parse)(input)
// }

// fn parse_alpha_or_spaces(input: &str) -> IResult<&str, String> {
//     map_res(alt((alpha1, space1)), str::parse)(input)
// }


// fn map_text_to_patch_type(input: &str) -> Option<PatchType> {
//     match input {
//         "SYNT" => Some(PatchType::Synth),
//         "KIT" => Some(PatchType::Kit),
//         _ => None,
//     }
// }

// fn parse_patch_type(input: &str) -> IResult<&str, Option<PatchType>> {
//     let (input, patch_type) = parse_alpha(input)?;

//     Ok((input, map_text_t o_patch_type(&patch_type)))
// }

// fn parse_number(input: &str) -> IResult<&str, u16> {
//     map_res(digit1, str::parse)(input)
// }

// fn parse_number_after_space(input: &str) -> IResult<&str, u16> {
//     let (input, _spaces) = space1(input)?;

//     map_res(digit1, str::parse)(input)
// }

// fn parse_standard_patch_name(input: &str) -> IResult<&str, PatchName> {
//     let (input, name) = parse_alpha(input)?;
//     let (input, number) = opt(parse_number)(input)?;
//     let (input, suffix) = opt(parse_alpha)(input)?;
//     let patch_type = PatchType::from_str(&name);
//     let result = match patch_type {
//         Ok(patch_type) => PatchName::Standard { patch_type, number: number.expect("patch number"), suffix: suffix.clone() },
//         Err(()) => PatchName::CustomName { name: name.to_string(), number },
//     };

//     Ok((input, result))
// }


// fn parse_custom_patch_name(input: &str) -> IResult<&str, PatchName> {
//     let (input, name) = parse_alpha_or_spaces(input)?;
//     let (input, number) = opt(parse_number_after_space)(input)?;

//     Ok((input, PatchName::CustomName { name, number }))
// }



#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("KIT000", PatchType::Kit, 0, None ; "KIT000")]
    #[test_case("KIT000A", PatchType::Kit, 0, Some('A') ; "KIT000A")]
    #[test_case("SYNT123V", PatchType::Synth, 123, Some('V') ; "SYNT123V")]
    fn parse_valid_input_standard_test(input: &str, expected_patch_type: PatchType, expected_number: u16, expected_suffix: Option<char>) {
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
        let expected = PatchName::CustomName {
            name: expected_name.to_string(),
            number: expected_number,
        };

        assert_eq!(expected, PatchName::from_str(input).unwrap());
    }
}
