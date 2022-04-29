//! Store On/Off value
//! The value is serialized as an integer where 0 means Off and anything else means On.
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OnOff {
    On,
    Off,
}

impl Default for OnOff {
    fn default() -> Self {
        OnOff::Off
    }
}

impl Serialize for OnOff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OnOff::On => serializer.serialize_u32(1u32),
            OnOff::Off => serializer.serialize_u32(0u32),
        }
    }
}

impl<'de> Deserialize<'de> for OnOff {
    fn deserialize<D>(deserializer: D) -> Result<OnOff, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i8(OnOffVisitor)
    }
}

struct OnOffVisitor;

impl<'de> Visitor<'de> for OnOffVisitor {
    type Value = OnOff;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("a number")
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match v {
            0i8 => OnOff::Off,
            _ => OnOff::On,
        })
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match v {
            0u8 => OnOff::Off,
            _ => OnOff::On,
        })
    }
}

impl std::fmt::Display for OnOff {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            OnOff::On => write!(f, "On"),
            OnOff::Off => write!(f, "Off"),
        }
    }
}
