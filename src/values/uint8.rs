//! 2022 12th march
//! I copy pasted from https://gist.github.com/franziskuskiefer/920fa6fdcf3d47cdbbdbe325e8e87275 then started over.
//! I found it by searching information about ranged integer. I found this issue still open:
//! https://github.com/rust-lang/rfcs/issues/671
//! Maybe one day this code will be useless!

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Uint8<const MIN: u8, const MAX: u8, const DEFAULT: u8> {
    val: u8,
}

impl<const MIN: u8, const MAX: u8, const DEFAULT: u8> From<u8> for Uint8<MIN, MAX, DEFAULT> {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl<const MIN: u8, const MAX: u8, const DEFAULT: u8> Default for Uint8<MIN, MAX, DEFAULT> {
    fn default() -> Self {
        Self::new(MIN)
    }
}

impl<const MIN: u8, const MAX: u8, const DEFAULT: u8> Uint8<MIN, MAX, DEFAULT> {
    const MIN: u8 = MIN;
    const MAX: u8 = MAX;

    fn check(val: u8) -> Self {
        debug_assert!(
            val >= Self::MIN && val <= Self::MAX,
            "{} <= {} <= {}",
            Self::MIN,
            val,
            Self::MAX
        );
        Self { val }
    }

    pub fn new(val: u8) -> Self {
        Self::check(val)
    }

    pub fn to_value(self) -> u8 {
        self.val
    }
}

impl<const MIN: u8, const MAX: u8, const DEFAULT: u8> Serialize for Uint8<MIN, MAX, DEFAULT> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.val)
    }
}

struct Uint8Visitor<const MIN: u8, const MAX: u8, const DEFAULT: u8>;

impl<'de, const MIN: u8, const MAX: u8, const DEFAULT: u8> Visitor<'de> for Uint8Visitor<MIN, MAX, DEFAULT> {
    type Value = Uint8<MIN, MAX, DEFAULT>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            formatter,
            "a string with an unsigned 8-bits decimal integer in range [{}; {}]",
            MIN, MAX
        )
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v > MAX {
            return Err(E::custom(format!("value '{}' is too big, can't be greater than {}", v, MAX)));
        }
        if v < MIN {
            return Err(E::custom(format!("value '{}' is too small, can't be lesser than {}", v, MIN)));
        }

        Ok(Self::Value::new(v))
    }
}

impl<'de, const MIN: u8, const MAX: u8, const DEFAULT: u8> Deserialize<'de> for Uint8<MIN, MAX, DEFAULT> {
    fn deserialize<D>(deserializer: D) -> Result<Uint8<MIN, MAX, DEFAULT>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(Uint8Visitor)
    }
}
