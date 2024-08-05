#[macro_export]
macro_rules! define_key_type {
    ($type_name:ident, $len:literal) => {
        #[derive(Clone)]
        pub struct $type_name {
            binary: [u8; $len],
            hex: std::sync::OnceLock<std::ffi::CString>,
        }

        impl std::hash::Hash for $type_name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.binary.hash(state);
            }
        }

        impl PartialOrd for $type_name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.binary.partial_cmp(&other.binary)
            }
        }

        impl Ord for $type_name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.binary.cmp(&other.binary)
            }
        }

        impl PartialEq for $type_name {
            fn eq(&self, other: &Self) -> bool {
                self.binary == other.binary
            }
        }

        impl Eq for $type_name {}

        impl crate::key::Key for $type_name {
            const LEN: usize = $len;
        }

        impl $type_name {
            pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
                use anyhow::Context;
                let mut binary = [0u8; $len];
                hex::decode_to_slice(hex, &mut binary)
                    .with_context(|| format!("Decoding hex {hex:?}"))?;

                let hex = std::sync::OnceLock::from(
                    std::ffi::CString::new(hex).context("unable to create cstring")?,
                );
                Ok(Self { binary, hex })
            }

            pub fn from_hex_cstring(hex: &std::ffi::CStr) -> anyhow::Result<Self> {
                use anyhow::Context;
                let hex = hex.to_str().context("Invalid hex")?;
                Self::from_hex(hex)
            }

            pub fn hex(&self) -> &str {
                self.hex_cstr()
                    .to_str()
                    .expect("string from hex to be valid utf8")
            }

            pub fn hex_cstr(&self) -> &std::ffi::CStr {
                self.hex
                    .get_or_init(|| {
                        std::ffi::CString::new(hex::encode(&self.binary))
                            .expect("unable to create cstring")
                    })
                    .as_c_str()
            }

            pub fn into_binary(self) -> [u8; $len] {
                self.binary
            }
        }

        impl From<[u8; $len]> for $type_name {
            fn from(binary: [u8; $len]) -> Self {
                Self {
                    binary,
                    hex: Default::default(),
                }
            }
        }

        impl Into<[u8; $len]> for &$type_name {
            fn into(self) -> [u8; $len] {
                self.binary
            }
        }

        impl<'a> TryFrom<&'a [u8]> for $type_name {
            type Error = anyhow::Error;

            fn try_from(slice: &'a [u8]) -> Result<Self, Self::Error> {
                if slice.len() != $len {
                    anyhow::bail!("Invalid length");
                }

                let mut binary = [0u8; $len];
                binary.copy_from_slice(slice);
                Ok(Self::from(binary))
            }
        }

        impl AsRef<[u8]> for $type_name {
            fn as_ref(&self) -> &[u8] {
                &self.binary
            }
        }

        impl std::ops::Deref for $type_name {
            type Target = [u8];

            fn deref(&self) -> &[u8] {
                &self.binary
            }
        }

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(self.hex(), f)
            }
        }

        impl std::fmt::Debug for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self.hex(), f)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type_name {
            fn deserialize<D>(deserializer: D) -> Result<$type_name, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s: String = serde::Deserialize::deserialize(deserializer)?;
                $type_name::from_hex(&s).map_err(serde::de::Error::custom)
            }
        }

        impl serde::Serialize for $type_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.hex().serialize(serializer)
            }
        }
    };
}

pub trait Key: AsRef<[u8]> {
    const LEN: usize;
}
