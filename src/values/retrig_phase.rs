//! Specify the phase in degrees.
//! This type is formatted as 32-bits unsigned integer hexadecimal.
//! Notice RetrigPhase(0) is different than RetrigPhase::Off!
use crate::values::{map_i32_u32, map_u32_i32, read_i32, Error};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::num::Wrapping;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum RetrigPhase {
    /// The phase is never reset
    Off,
    /// Degrees that the phase will be reset on note-on
    Degrees(u16),
}

impl RetrigPhase {
    const MAX_DEGREES: u16 = 360;

    pub fn new(degrees: u16) -> Self {
        Self::Degrees(degrees).normalise()
    }

    pub fn normalise(self) -> Self {
        match self {
            Self::Degrees(value) => {
                let mut value = value;

                if value > Self::MAX_DEGREES {
                    while value >= Self::MAX_DEGREES {
                        value -= Self::MAX_DEGREES;
                    }
                }

                Self::Degrees(value)
            }
            Self::Off => Self::Off,
        }
    }
}

impl Default for RetrigPhase {
    /// Create a phase set to Off
    fn default() -> Self {
        Self::Off
    }
}

impl std::fmt::Display for RetrigPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            RetrigPhase::Off => write!(f, "Off"),
            RetrigPhase::Degrees(degrees) => write!(f, "{}", degrees),
        }
    }
}

impl Serialize for RetrigPhase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = write_phase(self.normalise()).map_err(serde::ser::Error::custom)?;

        serializer.serialize_str(&value)
    }
}

struct PhaseVisitor;

impl<'de> Visitor<'de> for PhaseVisitor {
    type Value = RetrigPhase;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("a string with a signed 32-bits decimal integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        read_phase(v).map_err(|e| E::custom(e))
    }
}

impl<'de> Deserialize<'de> for RetrigPhase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PhaseVisitor)
    }
}

const PHASE_FACTOR: i32 = 11930464i32;
const PHASE_OFF_VALUE: &str = "-1";

fn write_phase(phase: RetrigPhase) -> Result<String, Error> {
    Ok(match phase {
        RetrigPhase::Off => PHASE_OFF_VALUE.to_string(),
        RetrigPhase::Degrees(value) => {
            let i32_value = Wrapping(map_u32_i32(value as u32)?);
            let result = i32_value * Wrapping(PHASE_FACTOR);

            result.0.to_string()
        }
    })
}

fn read_phase(text: &str) -> Result<RetrigPhase, Error> {
    let number = read_i32(text)?;
    let u32_value = map_i32_u32(number)?;

    Ok(match number {
        -1 => RetrigPhase::Off,
        _ => RetrigPhase::Degrees((u32_value / PHASE_FACTOR as u32) as u16),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(RetrigPhase::Off, RetrigPhase::Off ; "Off")]
    #[test_case(RetrigPhase::Degrees(0) , RetrigPhase::Degrees(0) ; "0")]
    #[test_case(RetrigPhase::Degrees(360) , RetrigPhase::Degrees(360) ; "360")]
    #[test_case(RetrigPhase::Degrees(361) , RetrigPhase::Degrees(1) ; "361")]
    #[test_case(RetrigPhase::Degrees(720) , RetrigPhase::Degrees(0) ; "720")]
    fn test_normalise_phase(input: RetrigPhase, expected: RetrigPhase) {
        assert_eq!(expected, input.normalise());
    }

    #[test_case(RetrigPhase::Off, "-1"; "Off")]
    #[test_case(RetrigPhase::Degrees(0), "0"; "0")]
    #[test_case(RetrigPhase::Degrees(1), "11930464"; "1")]
    #[test_case(RetrigPhase::Degrees(2), "23860928"; "2")]
    #[test_case(RetrigPhase::Degrees(10), "119304640"; "10")]
    #[test_case(RetrigPhase::Degrees(47), "560731808"; "47")]
    #[test_case(RetrigPhase::Degrees(179), "2135553056"; "179")]
    #[test_case(RetrigPhase::Degrees(180), "2147483520"; "180")]
    #[test_case(RetrigPhase::Degrees(181), "-2135553312"; "181")]
    #[test_case(RetrigPhase::Degrees(359), "-11930720"; "359")]
    #[test_case(RetrigPhase::Degrees(360), "-256"; "360")]
    fn test_write_phase(input: RetrigPhase, expected: &str) {
        assert_eq!(expected, write_phase(input).unwrap());
    }

    #[test_case(RetrigPhase::Off, "-1"; "Off")]
    #[test_case(RetrigPhase::Degrees(0), "0"; "0")]
    #[test_case(RetrigPhase::Degrees(1), "11930464"; "1")]
    #[test_case(RetrigPhase::Degrees(2), "23860928"; "2")]
    #[test_case(RetrigPhase::Degrees(10), "119304640"; "10")]
    #[test_case(RetrigPhase::Degrees(47), "560731808"; "47")]
    #[test_case(RetrigPhase::Degrees(179), "2135553056"; "179")]
    #[test_case(RetrigPhase::Degrees(180), "2147483520"; "180")]
    #[test_case(RetrigPhase::Degrees(181), "-2135553312"; "181")]
    #[test_case(RetrigPhase::Degrees(359), "-11930720"; "359")]
    #[test_case(RetrigPhase::Degrees(360), "-256"; "360")]
    fn test_read_phase(expected: RetrigPhase, input: &str) {
        assert_eq!(expected, read_phase(input).unwrap());
    }

    #[test_case(RetrigPhase::Off ; "Off")]
    #[test_case(RetrigPhase::Degrees(0) ; "0")]
    #[test_case(RetrigPhase::Degrees(1) ; "1")]
    #[test_case(RetrigPhase::Degrees(2) ; "2")]
    #[test_case(RetrigPhase::Degrees(10) ; "10")]
    #[test_case(RetrigPhase::Degrees(47) ; "47")]
    #[test_case(RetrigPhase::Degrees(179) ; "179")]
    #[test_case(RetrigPhase::Degrees(180) ; "180")]
    #[test_case(RetrigPhase::Degrees(181) ; "181")]
    #[test_case(RetrigPhase::Degrees(359) ; "359")]
    #[test_case(RetrigPhase::Degrees(360) ; "360")]
    fn test_write_read(input: RetrigPhase) {
        let string_representation = write_phase(input).unwrap();

        assert_eq!(input, read_phase(&string_representation).unwrap());
    }
}
