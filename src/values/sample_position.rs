use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplePosition(u64);

impl SamplePosition {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u32> for SamplePosition {
    fn from(position: u32) -> Self {
        Self::new(position as u64)
    }
}

impl From<u64> for SamplePosition {
    fn from(position: u64) -> Self {
        Self::new(position)
    }
}
