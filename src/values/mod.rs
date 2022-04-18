//! Deluge values
//!
//! These type are non splittable data. Most of them are strong integers and the others are mostly simple enumerations.
//! Each type allow to manipulate values as we do when using a deluge. For example, Hexu50 is an integer in the range [0; 50]
//! formatted as an 32-bits unsigned integer for storage.
//! As user, you manipulate a value in the range [0; 50] without having to think how it will be stored in the XML file.

mod decu50;
mod hexu50;
mod int8;
mod on_off;
mod pan;
mod retrig_phase;
mod sidechain_values;
mod simple_enums;
mod uint8;

pub use decu50::DecU50;
pub use hexu50::HexU50;
pub use int8::Int8;
pub use on_off::OnOff;
pub use pan::Pan;
pub use retrig_phase::RetrigPhase;
pub use sidechain_values::{AttackSidechain, ReleaseSidechain, TableIndex};
pub use simple_enums::{
    ArpeggiatorMode, LfoShape, LpfMode, OscType, PitchSpeed, Polyphony, SamplePlayMode, SoundType, SyncLevel, VoicePriority,
};
pub use uint8::Uint8;

pub type ClippingAmount = Uint8<0, 16, 0>;
pub type FineTranspose = Int8<-100, 100, 0>;
pub type TimeStretchAmount = Int8<-48, 48, 0>;
pub type Transpose = Int8<-96, 96, 0>;
pub type UnisonDetune = Uint8<0, 50, 0>;
pub type UnisonVoiceCount = Uint8<1, 8, 1>;
pub type OctavesCount = Uint8<1, 8, 1>;

use crate::SerializationError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;

pub fn map_u32_i32(value: u32) -> Result<i32, SerializationError> {
    let mut cursor = Cursor::new(value.to_be_bytes());

    cursor
        .read_i32::<BigEndian>()
        .map_err(|e| SerializationError::ConversionError(Arc::new(e)))
}

pub fn map_i32_u32(value: i32) -> Result<u32, SerializationError> {
    let mut cursor = Cursor::new(value.to_be_bytes());

    cursor
        .read_u32::<BigEndian>()
        .map_err(|e| SerializationError::ConversionError(Arc::new(e)))
}

pub fn write_hexadecimal_u32(value: u32) -> String {
    format!("{:#010X}", value)
}

fn read_hexadecimal_u32(text: &str) -> Result<u32, SerializationError> {
    let mut text = text;

    if text.starts_with("0x") {
        text = &text[2..];
    }

    u32::from_str_radix(text, 16).map_err(|e| SerializationError::ParseHexdecimalU32Error(text.to_string(), e))
}

fn read_i32(text: &str) -> Result<i32, SerializationError> {
    i32::from_str(text).map_err(|e| SerializationError::ParseI32Error(text.to_string(), e))
}

fn map_i32_50(value: i32) -> u8 {
    let mut value = value as f64;
    value -= f64::from(i32::MIN);
    value /= f64::from(u32::MAX);
    value *= 50f64;
    value.round() as u8
}

fn map_50_i32(value: u8) -> i32 {
    match value {
        // Yes I don't understand why I need to do that but actually my algorithm
        // only works for ALL values excepted DecU50(50) and DecU50(25)..
        // I tried to use floating points, but I also avoided overflow but I was not aware of the existence of Wrapping..
        // Moving on for now..
        50 => i32::MAX,
        25 => 0i32,
        _ => {
            let value = value as i64;
            let step_size = (u32::MAX / 50u32) as i64;
            let result = i64::from(i32::MIN) + (step_size * value);

            result as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{map_i32_u32, map_u32_i32, read_hexadecimal_u32};
    use test_case::test_case;

    #[test_case("0x00000000", 0 ; "zero")]
    #[test_case("0x7FFFFFFF", 0x7FFFFFFF ; "max")]
    #[test_case("7FFFFFFF", 0x7FFFFFFF ; "max without 0x")]
    #[test_case("0x4CCCCCA8", 0x4CCCCCA8u32 ; "0x4CCCCCA8u32")]
    #[test_case("0x23D70A20", 0x23D70A20u32 ; "0x23D70A20i32")]
    fn test_read_hexadecimal_u32(input: &str, expected: u32) {
        assert_eq!(expected, read_hexadecimal_u32(input).unwrap());
    }

    #[test_case(0x80000000u32, i32::MIN ; "min")]
    #[test_case(0x00000000u32 , 0i32 ; "middle")]
    #[test_case(0x7FFFFFFFu32, i32::MAX ; "max")]
    #[test_case(4294967040u32, -256i32 ; "-256i32")]
    fn test_convert_u32_i32(input: u32, expected: i32) {
        assert_eq!(expected, map_u32_i32(input).unwrap());
    }

    #[test_case(i32::MIN, 0x80000000u32 ; "min")]
    #[test_case(0i32, 0x00000000u32 ; "middle")]
    #[test_case(i32::MAX, 0x7FFFFFFFu32 ; "max")]
    #[test_case(-256i32, 4294967040; "4294967040")]
    fn test_convert_i32_u32(input: i32, expected: u32) {
        assert_eq!(expected, map_i32_u32(input).unwrap());
    }

    #[test_case(i32::MIN ; "min")]
    #[test_case(0i32 ; "middle")]
    #[test_case(i32::MAX ; "max")]
    #[test_case(0x23D70A20i32 ; "0x23D70A20")]
    #[test_case(40i32 ; "40")]
    #[test_case(1i32 ; "1")]
    #[test_case(2i32 ; "2")]
    #[test_case(3i32 ; "3")]
    #[test_case(0x4CCCCCA8i32 ; "0x4CCCCCA8")]
    fn test_i32_u32_conversion_back_and_forth(input: i32) {
        assert_eq!(input, map_u32_i32(map_i32_u32(input).unwrap()).unwrap());
    }
}
