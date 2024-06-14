use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct Hex<const LEN: usize>(pub [u8; LEN]);

impl AsRef<[u8]> for Hex<32> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for Hex<32> {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hex<32> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const LEN: usize> Display for Hex<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl<const LEN: usize> Debug for Hex<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<const LEN: usize> Serialize for Hex<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        hex::encode(&self.0).serialize(serializer)
    }
}

impl<'de, const LEN: usize> Deserialize<'de> for Hex<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        let mut array = [0u8; LEN];
        if bytes.len() != array.len() {
            return Err(serde::de::Error::custom(format!("Invalid length: {}, expected {}", bytes.len(), LEN)));
        }

        array.copy_from_slice(&bytes);
        Ok(Hex(array))
    }
}