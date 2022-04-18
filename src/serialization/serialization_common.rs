pub const LATEST_SUPPORTED_FIRMWARE_VERSION: &str = "3.1.5";

const DELUGE_SAMPLE_FREQUECY_RATE: u64 = 44100u64;

pub fn convert_milliseconds_to_samples(milliseconds: u64) -> u64 {
    milliseconds / DELUGE_SAMPLE_FREQUECY_RATE / 1000u64
}
