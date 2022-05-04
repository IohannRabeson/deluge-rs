//! Store an unsigned integer in the range [0; 50].
//! This type of value is formatted as an 32-bits unsigned integer hexadecimal.
use crate::values::{
    SerializationError, {map_50_i32, map_i32_50, map_i32_u32, map_u32_i32, read_hexadecimal_u32, write_hexadecimal_u32},
};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexU50(u8);

impl HexU50 {
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn parse(text: &str) -> Result<Self, SerializationError> {
        read_hexu50(text)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl From<u8> for HexU50 {
    fn from(value: u8) -> Self {
        HexU50::new(value)
    }
}

impl Serialize for HexU50 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = write_hexu50(*self).map_err(serde::ser::Error::custom)?;

        serializer.serialize_str(&value)
    }
}

struct HexU50Visitor;

impl<'de> Visitor<'de> for HexU50Visitor {
    type Value = HexU50;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("a string with unsigned hexadecimal number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        read_hexu50(v).map_err(|e| E::custom(e))
    }
}

impl<'de> Deserialize<'de> for HexU50 {
    fn deserialize<D>(deserializer: D) -> Result<HexU50, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexU50Visitor)
    }
}

impl std::fmt::Display for HexU50 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

fn map_i32_hexu50(value: i32) -> HexU50 {
    HexU50(map_i32_50(value))
}

fn map_hexu50_i32(value: HexU50) -> i32 {
    map_50_i32(value.0)
}

/// Read a 0-50 value encoded as unsigned u32 hexadecimal
fn read_hexu50(text: &str) -> Result<HexU50, SerializationError> {
    read_hexadecimal_u32(text).and_then(map_u32_i32).map(map_i32_hexu50)
}

/// Write a 0-50 value encoded as unsigned u32 hexadecimal with prefix 0x
/// The value must be in the interval [0; 50] or Error::Overflow and Error::Underflow are returned.
fn write_hexu50(value: HexU50) -> Result<String, SerializationError> {
    let value = map_hexu50_i32(value);
    let value = map_i32_u32(value)?;

    Ok(write_hexadecimal_u32(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(HexU50(0) , "0x80000000"; "0")]
    #[test_case(HexU50(1) , "0x851EB851"; "1")]
    #[test_case(HexU50(2) , "0x8A3D70A2"; "2")]
    #[test_case(HexU50(3) , "0x8F5C28F3"; "3")]
    #[test_case(HexU50(4) , "0x947AE144"; "4")]
    #[test_case(HexU50(5) , "0x99999995"; "5")]
    #[test_case(HexU50(6) , "0x9EB851E6"; "6")]
    #[test_case(HexU50(7) , "0xA3D70A37"; "7")]
    #[test_case(HexU50(8) , "0xA8F5C288"; "8")]
    #[test_case(HexU50(9) , "0xAE147AD9"; "9")]
    #[test_case(HexU50(10) , "0xB333332A"; "10")]
    #[test_case(HexU50(11) , "0xB851EB7B"; "11")]
    #[test_case(HexU50(12) , "0xBD70A3CC"; "12")]
    #[test_case(HexU50(13) , "0xC28F5C1D"; "13")]
    #[test_case(HexU50(14) , "0xC7AE146E"; "14")]
    #[test_case(HexU50(15) , "0xCCCCCCBF"; "15")]
    #[test_case(HexU50(16) , "0xD1EB8510"; "16")]
    #[test_case(HexU50(17) , "0xD70A3D61"; "17")]
    #[test_case(HexU50(18) , "0xDC28F5B2"; "18")]
    #[test_case(HexU50(19) , "0xE147AE03"; "19")]
    #[test_case(HexU50(20) , "0xE6666654"; "20")]
    #[test_case(HexU50(21) , "0xEB851EA5"; "21")]
    #[test_case(HexU50(22) , "0xF0A3D6F6"; "22")]
    #[test_case(HexU50(23) , "0xF5C28F47"; "23")]
    #[test_case(HexU50(24) , "0xFAE14798"; "24")]
    #[test_case(HexU50(25) , "0x00000000"; "25")]
    #[test_case(HexU50(26) , "0x051EB83A"; "26")]
    #[test_case(HexU50(27) , "0x0A3D708B"; "27")]
    #[test_case(HexU50(28) , "0x0F5C28DC"; "28")]
    #[test_case(HexU50(29) , "0x147AE12D"; "29")]
    #[test_case(HexU50(30) , "0x1999997E"; "30")]
    #[test_case(HexU50(31) , "0x1EB851CF"; "31")]
    #[test_case(HexU50(32) , "0x23D70A20"; "32")]
    #[test_case(HexU50(33) , "0x28F5C271"; "33")]
    #[test_case(HexU50(34) , "0x2E147AC2"; "34")]
    #[test_case(HexU50(35) , "0x33333313"; "35")]
    #[test_case(HexU50(36) , "0x3851EB64"; "36")]
    #[test_case(HexU50(37) , "0x3D70A3B5"; "37")]
    #[test_case(HexU50(38) , "0x428F5C06"; "38")]
    #[test_case(HexU50(39) , "0x47AE1457"; "39")]
    #[test_case(HexU50(40) , "0x4CCCCCA8"; "40")]
    #[test_case(HexU50(41) , "0x51EB84F9"; "41")]
    #[test_case(HexU50(42) , "0x570A3D4A"; "42")]
    #[test_case(HexU50(43) , "0x5C28F59B"; "43")]
    #[test_case(HexU50(44) , "0x6147ADEC"; "44")]
    #[test_case(HexU50(45) , "0x6666663D"; "45")]
    #[test_case(HexU50(46) , "0x6B851E8E"; "46")]
    #[test_case(HexU50(47) , "0x70A3D6DF"; "47")]
    #[test_case(HexU50(48) , "0x75C28F30"; "48")]
    #[test_case(HexU50(49) , "0x7AE14781"; "49")]
    #[test_case(HexU50(50) , "0x7FFFFFFF"; "50")]
    fn test_write_hexu50(input: HexU50, expected: &str) {
        assert_eq!(expected, write_hexu50(input).unwrap());
    }

    #[test_case("0x80000000" , HexU50(0); "0")]
    #[test_case("0x851EB851" , HexU50(1); "1")]
    #[test_case("0x8A3D70A2" , HexU50(2); "2")]
    #[test_case("0x8F5C28F3" , HexU50(3); "3")]
    #[test_case("0x947AE144" , HexU50(4); "4")]
    #[test_case("0x99999995" , HexU50(5); "5")]
    #[test_case("0x9EB851E6" , HexU50(6); "6")]
    #[test_case("0xA3D70A37" , HexU50(7); "7")]
    #[test_case("0xA8F5C288" , HexU50(8); "8")]
    #[test_case("0xAE147AD9" , HexU50(9); "9")]
    #[test_case("0xB333332A" , HexU50(10); "10")]
    #[test_case("0xB851EB7B" , HexU50(11); "11")]
    #[test_case("0xBD70A3CC" , HexU50(12); "12")]
    #[test_case("0xC28F5C1D" , HexU50(13); "13")]
    #[test_case("0xC7AE146E" , HexU50(14); "14")]
    #[test_case("0xCCCCCCBF" , HexU50(15); "15")]
    #[test_case("0xD1EB8510" , HexU50(16); "16")]
    #[test_case("0xD70A3D61" , HexU50(17); "17")]
    #[test_case("0xDC28F5B2" , HexU50(18); "18")]
    #[test_case("0xE147AE03" , HexU50(19); "19")]
    #[test_case("0xE6666654" , HexU50(20); "20")]
    #[test_case("0xEB851EA5" , HexU50(21); "21")]
    #[test_case("0xF0A3D6F6" , HexU50(22); "22")]
    #[test_case("0xF5C28F47" , HexU50(23); "23")]
    #[test_case("0xFAE14798" , HexU50(24); "24")]
    #[test_case("0x00000000" , HexU50(25); "25")]
    #[test_case("0x051EB83A" , HexU50(26); "26")]
    #[test_case("0x0A3D708B" , HexU50(27); "27")]
    #[test_case("0x0F5C28DC" , HexU50(28); "28")]
    #[test_case("0x147AE12D" , HexU50(29); "29")]
    #[test_case("0x1999997E" , HexU50(30); "30")]
    #[test_case("0x1EB851CF" , HexU50(31); "31")]
    #[test_case("0x23D70A20" , HexU50(32); "32")]
    #[test_case("0x28F5C271" , HexU50(33); "33")]
    #[test_case("0x2E147AC2" , HexU50(34); "34")]
    #[test_case("0x33333313" , HexU50(35); "35")]
    #[test_case("0x3851EB64" , HexU50(36); "36")]
    #[test_case("0x3D70A3B5" , HexU50(37); "37")]
    #[test_case("0x428F5C06" , HexU50(38); "38")]
    #[test_case("0x47AE1457" , HexU50(39); "39")]
    #[test_case("0x4CCCCCA8" , HexU50(40); "40")]
    #[test_case("0x51EB84F9" , HexU50(41); "41")]
    #[test_case("0x570A3D4A" , HexU50(42); "42")]
    #[test_case("0x5C28F59B" , HexU50(43); "43")]
    #[test_case("0x6147ADEC" , HexU50(44); "44")]
    #[test_case("0x6666663D" , HexU50(45); "45")]
    #[test_case("0x6B851E8E" , HexU50(46); "46")]
    #[test_case("0x70A3D6DF" , HexU50(47); "47")]
    #[test_case("0x75C28F30" , HexU50(48); "48")]
    #[test_case("0x7AE14781" , HexU50(49); "49")]
    #[test_case("0x7FFFFFFF" , HexU50(50); "50")]
    fn test_read_hexu50(input: &str, expected: HexU50) {
        assert_eq!(expected, read_hexu50(input).unwrap());
    }

    #[test]
    fn test_read_write_hexu50_40() {
        let value = HexU50(1);

        assert_eq!(value, read_hexu50(&write_hexu50(value).unwrap()).unwrap());
    }

    #[test_case("0x80000000" ; "0")]
    #[test_case("0x851EB851" ; "1")]
    #[test_case("0x00000000" ; "middle")]
    #[test_case("0x7FFFFFFF" ; "max")]
    #[test_case("0x4CCCCCA8" ; "0x4CCCCCA8u32")]
    fn test_read_write_hexu50(input: &str) {
        let h = read_hexu50(input).unwrap();
        eprintln!("hexu50: {}", h);

        assert_eq!(input, write_hexu50(h).unwrap());
    }

    #[test]
    fn test_map_hexu50_i32_50() {
        assert_eq!(2147483647i32, map_hexu50_i32(HexU50(50)));
        assert_eq!(2147483647u32, map_i32_u32(2147483647i32).unwrap());
        assert_eq!("0x7FFFFFFF", write_hexadecimal_u32(2147483647u32));
    }
}
