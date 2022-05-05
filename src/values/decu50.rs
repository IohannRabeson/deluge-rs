use crate::values::{map_50_i32, map_i32_50, read_i32, SerializationError};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DecU50(u8);

impl DecU50 {
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn parse(text: &str) -> Result<Self, SerializationError> {
        read_decu50(text)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl Serialize for DecU50 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = map_decu50_i32(*self);

        serializer.serialize_i32(value)
    }
}

struct DecU50Visitor;

impl<'de> Visitor<'de> for DecU50Visitor {
    type Value = DecU50;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("a string with unsigned hexadecimal number")
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(map_i32_decu50(v))
    }
}

impl<'de> Deserialize<'de> for DecU50 {
    fn deserialize<D>(deserializer: D) -> Result<DecU50, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i32(DecU50Visitor)
    }
}

impl std::fmt::Display for DecU50 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

fn map_i32_decu50(value: i32) -> DecU50 {
    DecU50(map_i32_50(value))
}

fn map_decu50_i32(value: DecU50) -> i32 {
    map_50_i32(value.0)
}

/// Read a 0-50 value encoded as unsigned u32 hexadecimal
fn read_decu50(text: &str) -> Result<DecU50, SerializationError> {
    read_i32(text).map(map_i32_decu50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("2147483647", DecU50(50); "50")]
    fn test_parse_decu50(input: &str, expected: DecU50) {
        assert_eq!(expected, DecU50::parse(input).unwrap());
    }
}
