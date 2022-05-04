//! Strong 8-bit integer constrained to a range defined at compile time.

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Int8<const MIN: i8, const MAX: i8, const DEFAULT: i8> {
    val: i8,
}

impl<const MIN: i8, const MAX: i8, const DEFAULT: i8> Int8<MIN, MAX, DEFAULT> {
    pub fn as_i8(&self) -> i8 {
        self.val
    }
}

impl<const MIN: i8, const MAX: i8, const DEFAULT: i8> Default for Int8<MIN, MAX, DEFAULT> {
    fn default() -> Self {
        Self::new(DEFAULT)
    }
}

impl<const MIN: i8, const MAX: i8, const DEFAULT: i8> From<i8> for Int8<MIN, MAX, DEFAULT> {
    fn from(value: i8) -> Self {
        Self::new(value)
    }
}

impl<const MIN: i8, const MAX: i8, const DEFAULT: i8> Int8<MIN, MAX, DEFAULT> {
    const MIN: i8 = MIN;
    const MAX: i8 = MAX;

    fn check(val: i8) -> Self {
        debug_assert!(
            val >= Self::MIN && val <= Self::MAX,
            "{} <= {} <= {}",
            Self::MIN,
            val,
            Self::MAX
        );
        Self { val }
    }

    pub fn new(val: i8) -> Self {
        Self::check(val)
    }
}

impl<const MIN: i8, const MAX: i8, const DEFAULT: i8> Serialize for Int8<MIN, MAX, DEFAULT> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(self.val)
    }
}

struct Uint8Visitor<const MIN: i8, const MAX: i8, const DEFAULT: i8>;

impl<'de, const MIN: i8, const MAX: i8, const DEFAULT: i8> Visitor<'de> for Uint8Visitor<MIN, MAX, DEFAULT> {
    type Value = Int8<MIN, MAX, DEFAULT>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            formatter,
            "a string with a signed 8-bits decimal integer in range [{}; {}]",
            MIN, MAX
        )
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
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

impl<'de, const MIN: i8, const MAX: i8, const DEFAULT: i8> Deserialize<'de> for Int8<MIN, MAX, DEFAULT> {
    fn deserialize<D>(deserializer: D) -> Result<Int8<MIN, MAX, DEFAULT>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i8(Uint8Visitor)
    }
}

impl<'de, const MIN: i8, const MAX: i8, const DEFAULT: i8> std::fmt::Display for Int8<MIN, MAX, DEFAULT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}
