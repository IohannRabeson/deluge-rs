use std::str::FromStr;
use nom::{
    character::complete::{alpha1, digit1},
    combinator::{map_res, opt},
    IResult, Finish,
};

pub type ParseError = nom::error::Error<String>;

/// A parsed patch name
/// ```
/// use deluge::PatchName;
/// use std::str::FromStr;
/// let patch_name = PatchName::from_str("SYNT234R").unwrap();
/// assert_eq!( 
///     patch_name,
///     PatchName{ name: "SYNT".to_string(), number: Some(234), suffix: Some("R".to_string()) },
/// )
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PatchName {
    pub name: String,
    pub number: Option<u16>,
    pub suffix: Option<String>,
}

impl FromStr for PatchName {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match parse_patch_name(input).finish() {
            Ok(patch_name) => Ok(patch_name.1),
            Err(nom::error::Error{ input, code }) => Err(nom::error::Error{ input: input.to_string(), code }),
        }
    }
}

impl ToString for PatchName {
    fn to_string(&self) -> String {
        let mut buffer = String::with_capacity(7);

        buffer.push_str(&self.name);
        
        if let Some(number) = self.number {
            buffer.push_str(&format!("{:03}", number));
        }

        if let Some(suffix) = &self.suffix {
            buffer.push_str(suffix);
        }

        buffer
    }
}

fn parse_alpha(input: &str) -> IResult<&str, String> {
    map_res(alpha1, str::parse)(input)
}

fn parse_number(input: &str) -> IResult<&str, u16> {
    map_res(digit1, str::parse)(input)
}

fn parse_patch_name(input: &str) -> IResult<&str, PatchName> {
    let (input, name) = parse_alpha(input)?;
    let (input, number) = opt(parse_number)(input)?;
    let (input, suffix) = opt(parse_alpha)(input)?;

    Ok((input, PatchName { name, number, suffix }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("KIT", "KIT", None, None ; "KIT")]
    #[test_case("KIT000", "KIT", Some(0), None ; "KIT000")]
    #[test_case("KIT000A", "KIT", Some(0), Some("A") ; "KIT000A")]
    fn parse_valid_input_test(input: &str, expected_name: &str, expected_number: Option<u16>, expected_suffix: Option<&str>) {
        let expected = PatchName {
            name: expected_name.to_string(),
            number: expected_number,
            suffix: expected_suffix.map(|text| text.to_string()),
        };

        assert_eq!(expected, PatchName::from_str(input).unwrap());
    }
}