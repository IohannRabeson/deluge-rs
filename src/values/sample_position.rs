use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SamplePosition(u64);

impl SamplePosition {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

impl Deref for SamplePosition {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for SamplePosition {
    fn from(position: u64) -> Self {
        Self::new(position)
    }
}
