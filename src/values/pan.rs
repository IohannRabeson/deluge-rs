//! Store a value in the range [-32L;32R].
//! The value is formatted as an 32-bits unsigned integer hexadecimal.

use crate::values::{map_i32_u32, map_u32_i32, read_hexadecimal_u32, write_hexadecimal_u32};
use crate::SerializationError;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Pan(i8);

impl Pan {
    const MAX_PAN: i8 = 32i8;
    const MIN_PAN: i8 = -32i8;

    pub fn new(value: i8) -> Result<Self, SerializationError> {
        if value > Self::MAX_PAN {
            return Err(SerializationError::Overflow(value.to_string(), Self::MAX_PAN.to_string()));
        }

        if value < Self::MIN_PAN {
            return Err(SerializationError::Underflow(value.to_string(), Self::MIN_PAN.to_string()));
        }

        Ok(Self(value))
    }

    pub fn parse(text: &str) -> Result<Self, SerializationError> {
        read_pan(text)
    }
}

impl std::fmt::Display for Pan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.0.cmp(&0) {
            std::cmp::Ordering::Less => write!(f, "L{}", -self.0),
            std::cmp::Ordering::Greater => write!(f, "R{}", self.0),
            std::cmp::Ordering::Equal => write!(f, "Center"),
        }
    }
}

const PAN_FACTOR: f64 = 67108864f64;

fn write_pan(pan: Pan) -> Result<String, SerializationError> {
    let value = (pan.0 as f64 * PAN_FACTOR) as i32;
    let value = map_i32_u32(value)?;

    Ok(write_hexadecimal_u32(value))
}

fn read_pan(text: &str) -> Result<Pan, SerializationError> {
    let number = read_hexadecimal_u32(text)?;
    let number = map_u32_i32(number)? as f64;

    Pan::new((number / PAN_FACTOR).round() as i8)
}

impl Serialize for Pan {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = write_pan(*self).map_err(serde::ser::Error::custom)?;

        serializer.serialize_str(&value)
    }
}

struct PanVisitor;

impl<'de> Visitor<'de> for PanVisitor {
    type Value = Pan;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("a string with unsigned hexadecimal number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        read_pan(v).map_err(|e| E::custom(e))
    }
}

impl<'de> Deserialize<'de> for Pan {
    fn deserialize<D>(deserializer: D) -> Result<Pan, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PanVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Pan::new(-32).unwrap(), "0x80000000" ; "0x80000000")]
    #[test_case(Pan::new(-31).unwrap(), "0x84000000" ; "0x84000000")]
    #[test_case(Pan::new(-30).unwrap(), "0x88000000" ; "0x88000000")]
    #[test_case(Pan::new(-29).unwrap(), "0x8C000000" ; "0x8C000000")]
    #[test_case(Pan::new(-28).unwrap(), "0x90000000" ; "0x90000000")]
    #[test_case(Pan::new(-27).unwrap(), "0x94000000" ; "0x94000000")]
    #[test_case(Pan::new(-26).unwrap(), "0x98000000" ; "0x98000000")]
    #[test_case(Pan::new(-25).unwrap(), "0x9C000000" ; "0x9C000000")]
    #[test_case(Pan::new(-24).unwrap(), "0xA0000000" ; "0xA0000000")]
    #[test_case(Pan::new(-23).unwrap(), "0xA4000000" ; "0xA4000000")]
    #[test_case(Pan::new(-22).unwrap(), "0xA8000000" ; "0xA8000000")]
    #[test_case(Pan::new(-21).unwrap(), "0xAC000000" ; "0xAC000000")]
    #[test_case(Pan::new(-20).unwrap(), "0xB0000000" ; "0xB0000000")]
    #[test_case(Pan::new(-19).unwrap(), "0xB4000000" ; "0xB4000000")]
    #[test_case(Pan::new(-18).unwrap(), "0xB8000000" ; "0xB8000000")]
    #[test_case(Pan::new(-17).unwrap(), "0xBC000000" ; "0xBC000000")]
    #[test_case(Pan::new(-16).unwrap(), "0xC0000000" ; "0xC0000000")]
    #[test_case(Pan::new(-15).unwrap(), "0xC4000000" ; "0xC4000000")]
    #[test_case(Pan::new(-14).unwrap(), "0xC8000000" ; "0xC8000000")]
    #[test_case(Pan::new(-13).unwrap(), "0xCC000000" ; "0xCC000000")]
    #[test_case(Pan::new(-12).unwrap(), "0xD0000000" ; "0xD0000000")]
    #[test_case(Pan::new(-11).unwrap(), "0xD4000000" ; "0xD4000000")]
    #[test_case(Pan::new(-10).unwrap(), "0xD8000000" ; "0xD8000000")]
    #[test_case(Pan::new(-9).unwrap(), "0xDC000000" ; "0xDC000000")]
    #[test_case(Pan::new(-8).unwrap(), "0xE0000000" ; "0xE0000000")]
    #[test_case(Pan::new(-7).unwrap(), "0xE4000000" ; "0xE4000000")]
    #[test_case(Pan::new(-6).unwrap(), "0xE8000000" ; "0xE8000000")]
    #[test_case(Pan::new(-5).unwrap(), "0xEC000000" ; "0xEC000000")]
    #[test_case(Pan::new(-4).unwrap(), "0xF0000000" ; "0xF0000000")]
    #[test_case(Pan::new(-3).unwrap(), "0xF4000000" ; "0xF4000000")]
    #[test_case(Pan::new(-2).unwrap(), "0xF8000000" ; "0xF8000000")]
    #[test_case(Pan::new(-1).unwrap(), "0xFC000000" ; "0xFC000000")]
    #[test_case(Pan::new(0).unwrap(), "0x00000000" ; "0x00000000")]
    #[test_case(Pan::new(1).unwrap(), "0x04000000" ; "0x04000000")]
    #[test_case(Pan::new(2).unwrap(), "0x08000000" ; "0x08000000")]
    #[test_case(Pan::new(3).unwrap(), "0x0C000000" ; "0x0C000000")]
    #[test_case(Pan::new(4).unwrap(), "0x10000000" ; "0x10000000")]
    #[test_case(Pan::new(5).unwrap(), "0x14000000" ; "0x14000000")]
    #[test_case(Pan::new(6).unwrap(), "0x18000000" ; "0x18000000")]
    #[test_case(Pan::new(7).unwrap(), "0x1C000000" ; "0x1C000000")]
    #[test_case(Pan::new(8).unwrap(), "0x20000000" ; "0x20000000")]
    #[test_case(Pan::new(9).unwrap(), "0x24000000" ; "0x24000000")]
    #[test_case(Pan::new(10).unwrap(), "0x28000000" ; "0x28000000")]
    #[test_case(Pan::new(11).unwrap(), "0x2C000000" ; "0x2C000000")]
    #[test_case(Pan::new(12).unwrap(), "0x30000000" ; "0x30000000")]
    #[test_case(Pan::new(13).unwrap(), "0x34000000" ; "0x34000000")]
    #[test_case(Pan::new(14).unwrap(), "0x38000000" ; "0x38000000")]
    #[test_case(Pan::new(15).unwrap(), "0x3C000000" ; "0x3C000000")]
    #[test_case(Pan::new(16).unwrap(), "0x40000000" ; "0x40000000")]
    #[test_case(Pan::new(17).unwrap(), "0x44000000" ; "0x44000000")]
    #[test_case(Pan::new(18).unwrap(), "0x48000000" ; "0x48000000")]
    #[test_case(Pan::new(19).unwrap(), "0x4C000000" ; "0x4C000000")]
    #[test_case(Pan::new(20).unwrap(), "0x50000000" ; "0x50000000")]
    #[test_case(Pan::new(21).unwrap(), "0x54000000" ; "0x54000000")]
    #[test_case(Pan::new(22).unwrap(), "0x58000000" ; "0x58000000")]
    #[test_case(Pan::new(23).unwrap(), "0x5C000000" ; "0x5C000000")]
    #[test_case(Pan::new(24).unwrap(), "0x60000000" ; "0x60000000")]
    #[test_case(Pan::new(25).unwrap(), "0x64000000" ; "0x64000000")]
    #[test_case(Pan::new(26).unwrap(), "0x68000000" ; "0x68000000")]
    #[test_case(Pan::new(27).unwrap(), "0x6C000000" ; "0x6C000000")]
    #[test_case(Pan::new(28).unwrap(), "0x70000000" ; "0x70000000")]
    #[test_case(Pan::new(29).unwrap(), "0x74000000" ; "0x74000000")]
    #[test_case(Pan::new(30).unwrap(), "0x78000000" ; "0x78000000")]
    #[test_case(Pan::new(31).unwrap(), "0x7C000000" ; "0x7C000000")]
    #[test_case(Pan::new(32).unwrap(), "0x7FFFFFFF" ; "0x7FFFFFFF")]
    fn test_read_pan(expected: Pan, input: &str) {
        assert_eq!(expected, read_pan(input).unwrap());
    }

    #[test_case(Pan::new(-32).unwrap(), "0x80000000" ; "0x80000000")]
    #[test_case(Pan::new(-31).unwrap(), "0x84000000" ; "0x84000000")]
    #[test_case(Pan::new(-30).unwrap(), "0x88000000" ; "0x88000000")]
    #[test_case(Pan::new(-29).unwrap(), "0x8C000000" ; "0x8C000000")]
    #[test_case(Pan::new(-28).unwrap(), "0x90000000" ; "0x90000000")]
    #[test_case(Pan::new(-27).unwrap(), "0x94000000" ; "0x94000000")]
    #[test_case(Pan::new(-26).unwrap(), "0x98000000" ; "0x98000000")]
    #[test_case(Pan::new(-25).unwrap(), "0x9C000000" ; "0x9C000000")]
    #[test_case(Pan::new(-24).unwrap(), "0xA0000000" ; "0xA0000000")]
    #[test_case(Pan::new(-23).unwrap(), "0xA4000000" ; "0xA4000000")]
    #[test_case(Pan::new(-22).unwrap(), "0xA8000000" ; "0xA8000000")]
    #[test_case(Pan::new(-21).unwrap(), "0xAC000000" ; "0xAC000000")]
    #[test_case(Pan::new(-20).unwrap(), "0xB0000000" ; "0xB0000000")]
    #[test_case(Pan::new(-19).unwrap(), "0xB4000000" ; "0xB4000000")]
    #[test_case(Pan::new(-18).unwrap(), "0xB8000000" ; "0xB8000000")]
    #[test_case(Pan::new(-17).unwrap(), "0xBC000000" ; "0xBC000000")]
    #[test_case(Pan::new(-16).unwrap(), "0xC0000000" ; "0xC0000000")]
    #[test_case(Pan::new(-15).unwrap(), "0xC4000000" ; "0xC4000000")]
    #[test_case(Pan::new(-14).unwrap(), "0xC8000000" ; "0xC8000000")]
    #[test_case(Pan::new(-13).unwrap(), "0xCC000000" ; "0xCC000000")]
    #[test_case(Pan::new(-12).unwrap(), "0xD0000000" ; "0xD0000000")]
    #[test_case(Pan::new(-11).unwrap(), "0xD4000000" ; "0xD4000000")]
    #[test_case(Pan::new(-10).unwrap(), "0xD8000000" ; "0xD8000000")]
    #[test_case(Pan::new(-9).unwrap(), "0xDC000000" ; "0xDC000000")]
    #[test_case(Pan::new(-8).unwrap(), "0xE0000000" ; "0xE0000000")]
    #[test_case(Pan::new(-7).unwrap(), "0xE4000000" ; "0xE4000000")]
    #[test_case(Pan::new(-6).unwrap(), "0xE8000000" ; "0xE8000000")]
    #[test_case(Pan::new(-5).unwrap(), "0xEC000000" ; "0xEC000000")]
    #[test_case(Pan::new(-4).unwrap(), "0xF0000000" ; "0xF0000000")]
    #[test_case(Pan::new(-3).unwrap(), "0xF4000000" ; "0xF4000000")]
    #[test_case(Pan::new(-2).unwrap(), "0xF8000000" ; "0xF8000000")]
    #[test_case(Pan::new(-1).unwrap(), "0xFC000000" ; "0xFC000000")]
    #[test_case(Pan::new(0).unwrap(), "0x00000000" ; "0x00000000")]
    #[test_case(Pan::new(1).unwrap(), "0x04000000" ; "0x04000000")]
    #[test_case(Pan::new(2).unwrap(), "0x08000000" ; "0x08000000")]
    #[test_case(Pan::new(3).unwrap(), "0x0C000000" ; "0x0C000000")]
    #[test_case(Pan::new(4).unwrap(), "0x10000000" ; "0x10000000")]
    #[test_case(Pan::new(5).unwrap(), "0x14000000" ; "0x14000000")]
    #[test_case(Pan::new(6).unwrap(), "0x18000000" ; "0x18000000")]
    #[test_case(Pan::new(7).unwrap(), "0x1C000000" ; "0x1C000000")]
    #[test_case(Pan::new(8).unwrap(), "0x20000000" ; "0x20000000")]
    #[test_case(Pan::new(9).unwrap(), "0x24000000" ; "0x24000000")]
    #[test_case(Pan::new(10).unwrap(), "0x28000000" ; "0x28000000")]
    #[test_case(Pan::new(11).unwrap(), "0x2C000000" ; "0x2C000000")]
    #[test_case(Pan::new(12).unwrap(), "0x30000000" ; "0x30000000")]
    #[test_case(Pan::new(13).unwrap(), "0x34000000" ; "0x34000000")]
    #[test_case(Pan::new(14).unwrap(), "0x38000000" ; "0x38000000")]
    #[test_case(Pan::new(15).unwrap(), "0x3C000000" ; "0x3C000000")]
    #[test_case(Pan::new(16).unwrap(), "0x40000000" ; "0x40000000")]
    #[test_case(Pan::new(17).unwrap(), "0x44000000" ; "0x44000000")]
    #[test_case(Pan::new(18).unwrap(), "0x48000000" ; "0x48000000")]
    #[test_case(Pan::new(19).unwrap(), "0x4C000000" ; "0x4C000000")]
    #[test_case(Pan::new(20).unwrap(), "0x50000000" ; "0x50000000")]
    #[test_case(Pan::new(21).unwrap(), "0x54000000" ; "0x54000000")]
    #[test_case(Pan::new(22).unwrap(), "0x58000000" ; "0x58000000")]
    #[test_case(Pan::new(23).unwrap(), "0x5C000000" ; "0x5C000000")]
    #[test_case(Pan::new(24).unwrap(), "0x60000000" ; "0x60000000")]
    #[test_case(Pan::new(25).unwrap(), "0x64000000" ; "0x64000000")]
    #[test_case(Pan::new(26).unwrap(), "0x68000000" ; "0x68000000")]
    #[test_case(Pan::new(27).unwrap(), "0x6C000000" ; "0x6C000000")]
    #[test_case(Pan::new(28).unwrap(), "0x70000000" ; "0x70000000")]
    #[test_case(Pan::new(29).unwrap(), "0x74000000" ; "0x74000000")]
    #[test_case(Pan::new(30).unwrap(), "0x78000000" ; "0x78000000")]
    #[test_case(Pan::new(31).unwrap(), "0x7C000000" ; "0x7C000000")]
    #[test_case(Pan::new(32).unwrap(), "0x7FFFFFFF" ; "0x7FFFFFFF")]
    fn test_write_pan(input: Pan, expected: &str) {
        assert_eq!(expected, write_pan(input).unwrap());
    }

    #[test]
    fn test_read_pan_32() {
        assert_eq!(Pan::new(32).unwrap(), read_pan("0x7FFFFFFF").unwrap());
    }
}
