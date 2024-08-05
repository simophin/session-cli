use std::{
    fmt::{Debug, Display},
    net::Ipv4Addr,
};

use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deref)]
pub struct PublicIPv4(Ipv4Addr);

impl AsRef<Ipv4Addr> for PublicIPv4 {
    fn as_ref(&self) -> &Ipv4Addr {
        &self.0
    }
}

impl Display for PublicIPv4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for PublicIPv4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl PublicIPv4 {
    pub fn new(ip: Ipv4Addr) -> Option<Self> {
        if ip.is_private()
            || ip.is_broadcast()
            || ip.is_documentation()
            || ip.is_link_local()
            || ip.is_multicast()
        {
            None
        } else {
            Some(Self(ip))
        }
    }
}

impl<'de> Deserialize<'de> for PublicIPv4 {
    fn deserialize<D>(deserializer: D) -> Result<PublicIPv4, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.parse() {
            Ok(ip) => Ok(Self::new(ip).ok_or_else(|| serde::de::Error::custom("Private IP"))?),
            Err(_) => Err(serde::de::Error::custom("Invalid IP address")),
        }
    }
}
