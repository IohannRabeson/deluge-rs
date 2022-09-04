//! Release and Release type for sidechain
//! I can't use an array as const generic parameter. But I guess this is something that can come one day.
//! For now, I resolve that by having a little bit of code duplicated (AttackSidechain and ReleaseSidechain only have differents numbers in their tables).
//!
use super::SerializationError;
use crate::values::Uint8;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::convert::From;

/// Type of a table index
pub type TableIndex = Uint8<0, 51, 0>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AttackSidechain {
    index: Uint8<0, 51, 0>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReleaseSidechain {
    index: Uint8<0, 51, 0>,
}

impl AttackSidechain {
    const SIDECHAIN_ATTACK_VALUES: [u32; 51] = [
        1048576, 887876, 751804, 636588, 539028, 456420, 386472, 327244, 277092, 234624, 198668, 168220, 142440, 120612, 102128,
        86476, 73224, 62000, 52500, 44452, 37640, 31872, 26988, 22852, 19348, 16384, 13876, 11748, 9948, 8428, 7132, 6040, 5112,
        4328, 3668, 3104, 2628, 2224, 1884, 1596, 1352, 1144, 968, 820, 696, 558, 496, 420, 356, 304, 256,
    ];

    pub fn new(index: TableIndex) -> Self {
        Self { index }
    }

    pub fn to_u32(self) -> u32 {
        Self::SIDECHAIN_ATTACK_VALUES[self.index.as_u8() as usize]
    }
}

impl From<TableIndex> for AttackSidechain {
    fn from(index: TableIndex) -> Self {
        Self::new(index)
    }
}

impl TryFrom<u32> for AttackSidechain {
    type Error = SerializationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match Self::SIDECHAIN_ATTACK_VALUES.binary_search_by(|probe| probe.cmp(&value).reverse()) {
            Ok(index) => Ok(Self::new(TableIndex::new(index as u8))),
            Err(_) => Err(Self::Error::ValueNotFoundInTable(value)),
        }
    }
}

impl Serialize for AttackSidechain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.to_u32())
    }
}

struct AttackSidechainVisitor;

impl<'de> Visitor<'de> for AttackSidechainVisitor {
    type Value = AttackSidechain;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("8 bits unsigned in range [0; 50]")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        AttackSidechain::try_from(v).map_err(|e| E::custom(e))
    }
}

impl<'de> Deserialize<'de> for AttackSidechain {
    fn deserialize<D>(deserializer: D) -> Result<AttackSidechain, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(AttackSidechainVisitor)
    }
}

impl ReleaseSidechain {
    const SIDECHAIN_RELEASE_VALUES: [u32; 51] = [
        261528, 38632, 19552, 13184, 9872, 7840, 6472, 5480, 4736, 4152, 3680, 3296, 2976, 2704, 2472, 2264, 2088, 1928, 1792,
        1664, 1552, 1448, 1352, 1272, 1192, 1120, 1056, 992, 936, 880, 832, 784, 744, 704, 664, 624, 592, 560, 528, 496, 472,
        448, 424, 400, 376, 352, 328, 312, 288, 272, 256,
    ];

    pub fn new(index: TableIndex) -> Self {
        Self { index }
    }

    pub fn to_u32(self) -> u32 {
        Self::SIDECHAIN_RELEASE_VALUES[self.index.as_u8() as usize]
    }
}

impl From<TableIndex> for ReleaseSidechain {
    fn from(index: TableIndex) -> Self {
        Self::new(index)
    }
}

impl TryFrom<u32> for ReleaseSidechain {
    type Error = SerializationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match Self::SIDECHAIN_RELEASE_VALUES.binary_search_by(|probe| probe.cmp(&value).reverse()) {
            Ok(index) => Ok(Self::new(TableIndex::new(index as u8))),
            Err(_) => Err(Self::Error::ValueNotFoundInTable(value)),
        }
    }
}

impl Serialize for ReleaseSidechain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.to_u32())
    }
}

struct ReleaseSidechainVisitor;

impl<'de> Visitor<'de> for ReleaseSidechainVisitor {
    type Value = ReleaseSidechain;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str("8 bits unsigned in range [0; 50]")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        ReleaseSidechain::try_from(v).map_err(|e| E::custom(e))
    }
}

impl<'de> Deserialize<'de> for ReleaseSidechain {
    fn deserialize<D>(deserializer: D) -> Result<ReleaseSidechain, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(ReleaseSidechainVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(1048576, AttackSidechain::from(TableIndex::new(0)) ; "min")]
    #[test_case(539028, AttackSidechain::from(TableIndex::new(4)) ; "539028")]
    #[test_case(256, AttackSidechain::from(TableIndex::new(50)) ; "max")]
    fn test_attack_sidechain_try_from(input: u32, expected: AttackSidechain) {
        assert_eq!(expected, AttackSidechain::try_from(input).unwrap());
    }
}
