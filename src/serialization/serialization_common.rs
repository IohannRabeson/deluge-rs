use crate::SerializationError;

pub const LATEST_SUPPORTED_FIRMWARE_VERSION: &str = "3.1.5";

pub fn parse_u8(input: &str) -> Result<u8, SerializationError> {
    input.parse::<u8>().map_err(SerializationError::ParseIntError)
}

// We really want this function takes a &String and not a &str to avoid a build error.
#[allow(clippy::ptr_arg)]
pub fn parse_u8_string(input: &String) -> Result<u8, SerializationError> {
    input.parse::<u8>().map_err(SerializationError::ParseIntError)
}

const DELUGE_SAMPLE_FREQUECY_RATE: u64 = 44100u64;

pub fn convert_milliseconds_to_samples(milliseconds: u64) -> u64 {
    milliseconds / DELUGE_SAMPLE_FREQUECY_RATE / 1000u64
}
