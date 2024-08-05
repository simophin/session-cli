use derive_more::Deref;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Deref)]
pub struct NonEmptyStringRef<'a>(&'a str);

impl<'a> Display for NonEmptyStringRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Debug for NonEmptyStringRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Serialize for NonEmptyStringRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a> NonEmptyStringRef<'a> {
    pub fn new(data: &'a str) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        Some(Self(data))
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
}
