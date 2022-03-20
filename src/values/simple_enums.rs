use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use serde_repr::*;

/// Polyphony
/// I noticed there are few patches have "0" or "1" as value.
/// SYNT184.XML and SYNT095.XML for example. I will have to implement an alternative serialization but
/// I will keep the attributes for the "latest" supported version.
///
/// Each times, it's for a FM patch. I'm quite sure internaly Subtractive synth and Fm synth are different structure.
#[derive(Clone, Serialize, PartialEq, Eq, Debug)]
pub enum Polyphony {
    #[serde(rename = "poly")]
    Poly,

    #[serde(rename = "mono")]
    Mono,

    #[serde(rename = "auto")]
    Auto,

    #[serde(rename = "legato")]
    Legato,

    #[serde(rename = "choke")]
    Choke,
}

struct PolyphonyVisitor;

impl<'de> Visitor<'de> for PolyphonyVisitor {
    type Value = Polyphony;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string with a polyphony")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "poly" => Ok(Self::Value::Poly),
            "mono" => Ok(Self::Value::Mono),
            "legato" => Ok(Self::Value::Legato),
            "choke" => Ok(Self::Value::Choke),
            "auto" => Ok(Self::Value::Auto),
            _ => get_polyphony_v1(v).ok_or_else(|| E::custom(format!("unsupported polyphony value format v1 '{}'", v))),
        }
    }
}

fn get_polyphony_v1(text: &str) -> Option<Polyphony> {
    Some(match text.parse::<u8>().ok()? {
        0u8 => Polyphony::Auto,
        1u8 => Polyphony::Poly,
        2u8 => Polyphony::Choke,
        _ => return None,
    })
}

impl<'de> Deserialize<'de> for Polyphony {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PolyphonyVisitor)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SoundType {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "subtractive")]
    Subtractive,
    #[serde(rename = "ringmod")]
    RingMod,
    #[serde(rename = "fm")]
    Fm,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum OscType {
    #[serde(rename = "square")]
    Square,
    #[serde(rename = "sine")]
    Sine,
    #[serde(rename = "saw")]
    Saw,
    #[serde(rename = "triangle")]
    Triangle,
    #[serde(rename = "analogSquare")]
    AnalogSquare,
    #[serde(rename = "analogSaw")]
    AnalogSaw,
    #[serde(rename = "sample")]
    Sample,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum LfoShape {
    #[serde(rename = "square")]
    Square,
    #[serde(rename = "sine")]
    Sine,
    #[serde(rename = "saw")]
    Saw,
    #[serde(rename = "triangle")]
    Triangle,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SamplePlayMode {
    Cut = 0,
    Once = 1,
    Loop = 2,
    Stretch = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PitchSpeed {
    Linked = 1,
    Independent = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SyncLevel {
    Off = 0,
    /// 4 bars
    FourBars = 1,
    /// 2 bars
    TwoBars = 2,
    /// 1 bar
    OneBar = 3,
    /// 2nd
    Second = 4,
    /// 4th
    Fourth = 5,
    /// 8th
    Eighth = 6,
    /// 16th
    Sixteenth = 7,
    /// 32th
    ThirtySecond = 8,
    /// 64th
    SixtyFourth = 9,
    /// 128th
    HundredTwentyEighth = 10,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum LpfMode {
    #[serde(rename = "24dB")]
    Lpf24,
    #[serde(rename = "12dB")]
    Lpf12,
    #[serde(rename = "24dBDrive")]
    Lpf24Drive,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ArpeggiatorMode {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "up")]
    Up,
    #[serde(rename = "down")]
    Down,
    #[serde(rename = "both")]
    Both,
    #[serde(rename = "random")]
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum VoicePriority {
    Low = 0,
    Medium = 1,
    High = 2,
}
