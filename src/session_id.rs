use crate::curve25519::Curve25519PubKey;
use crate::ed25519::ED25519PubKey;
use crate::key::Key;
use paste::paste;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::ffi::{c_char, CStr, CString};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::OnceLock;

macro_rules! define_id_type {
    ($name:ident, $backing:ty, $prefix:literal) => {
        #[derive(SerializeDisplay, DeserializeFromStr, Clone, PartialEq, Eq)]
        pub struct $name {
            backing_key: $backing,
            display: OnceLock<CString>,
        }

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.backing_key.hash(state);
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.backing_key.partial_cmp(&other.backing_key)
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.backing_key.cmp(&other.backing_key)
            }
        }

        impl $name {
            pub fn new(backing_key: $backing) -> Self {
                Self {
                    backing_key,
                    display: OnceLock::new(),
                }
            }

            pub fn from_c_string_array(arr: &[c_char; 67]) -> Option<Self> {
                let str = std::str::from_utf8(unsafe {
                    std::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len() - 1)
                })
                .ok()?;

                str.parse().ok()
            }

            pub fn as_str(&self) -> &str {
                self.as_c_str().to_str().unwrap()
            }

            pub fn pub_key(&self) -> &$backing {
                &self.backing_key
            }

            pub fn as_c_str(&self) -> &CStr {
                self.display
                    .get_or_init(|| {
                        CString::new(format!("{}{}", $prefix, self.backing_key.hex())).unwrap()
                    })
                    .as_c_str()
            }
        }

        impl From<$backing> for $name {
            fn from(backing_key: $backing) -> Self {
                Self::new(backing_key)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> anyhow::Result<Self> {
                if !s.starts_with($prefix) {
                    anyhow::bail!("Expecting prefix {} in string {:?}", $prefix, s);
                }

                let mut backing_key = [0u8; <$backing as Key>::LEN];
                hex::decode_to_slice(&s[$prefix.len()..], &mut backing_key)?;
                let backing_key = <$backing>::from(backing_key);
                let display = OnceLock::from(CString::new(s).unwrap());
                Ok(Self {
                    backing_key,
                    display,
                })
            }
        }
    };
}

define_id_type!(IndividualID, Curve25519PubKey, "05");
define_id_type!(GroupID, ED25519PubKey, "03");
define_id_type!(BlindedID, Curve25519PubKey, "15");

macro_rules! define_id_combinations {
    ($name:ident $(,$id_type:ident)+) => {
        paste! {
            #[derive(Clone, Debug, SerializeDisplay, DeserializeFromStr, Eq, PartialEq)]
            pub enum $name {
                $(
                    $id_type([<$id_type ID>]),
                )+
            }

            impl Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(self.as_str())
                }
            }

            $(
            impl From<[<$id_type ID>]> for $name {
                fn from(id: [<$id_type ID>]) -> Self {
                    Self::$id_type(id)
                }
            }
            )+

            $(
            impl TryFrom<$name> for [<$id_type ID>] {
                type Error = ();

                fn try_from(id: $name) -> Result<Self, Self::Error> {
                    match id {
                        $name::$id_type(id) => Ok(id),
                        _ => Err(()),
                    }
                }
            }
            )+

            impl $name {
                pub fn as_str(&self) -> &str {
                    match self {
                        $(
                            Self::$id_type(id) => id.as_str(),
                        )+
                    }
                }

                pub fn pub_key_bytes(&self) -> &[u8]  {
                    match self {
                        $(
                            Self::$id_type(id) => id.pub_key().as_ref(),
                        )+
                    }
                }
            }

            impl FromStr for $name {
                type Err = anyhow::Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    $(
                        if let Ok(id) = [<$id_type ID>]::from_str(s) {
                            return Ok($name::$id_type(id));
                        }
                    )+
                    anyhow::bail!("Invalid ID: {}", s)
                }
            }
        }
    };
}

define_id_combinations!(SessionID, Individual, Group, Blinded);
define_id_combinations!(IndividualOrBlindedID, Individual, Blinded);
define_id_combinations!(IndividualOrGroupID, Individual, Group);

macro_rules! define_session_id_conversion {
    ($target:ident $(,$variant:ident)+) => {
        impl From<$target> for SessionID {
            fn from(id: $target) -> Self {
                match id {
                    $( $target::$variant(id) => Self::$variant(id), )+
                }
            }
        }

        impl TryFrom<SessionID> for $target {
            type Error = ();

            fn try_from(id: SessionID) -> Result<Self, Self::Error> {
                match id {
                    $( SessionID::$variant(id) => Ok($target::$variant(id)), )+
                    _ => Err(()),
                }
            }
        }

        impl $target {
            pub fn to_session_id(&self) -> SessionID {
                match self {
                    $( Self::$variant(id) => SessionID::$variant(id.clone()), )+
                }
            }
        }
    };
}

define_session_id_conversion!(IndividualOrGroupID, Individual, Group);
define_session_id_conversion!(IndividualOrBlindedID, Individual, Blinded);
