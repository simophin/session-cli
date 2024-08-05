use crate::bindings;
use crate::clock::Timestamp;
use crate::config::ConfigExt;
use crate::utils::CArrayExt;
use anyhow::bail;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::ffi::CStr;
use url::Url;

impl crate::bindings::user_profile_pic {
    pub fn url(&self) -> Option<Url> {
        let url_str = self.url.cstr_to_str()?;
        url_str.parse().ok()
    }

    pub fn is_empty(&self) -> bool {
        self.url[0] == 0
    }
}

impl Serialize for crate::bindings::user_profile_pic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("UserProfilePic", 2)?;
        st.serialize_field("url", &self.url())?;
        st.serialize_field("key", self.key.as_slice())?;
        st.end()
    }
}

impl super::UserProfileConfig {
    pub fn profile_pic(&self) -> Option<crate::bindings::user_profile_pic> {
        Some(unsafe { crate::bindings::user_profile_get_pic(self.as_ref() as *const _) })
    }

    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(crate::bindings::user_profile_get_name(
                self.as_ref() as *const _
            ))
            .to_str()
            .unwrap_or_default()
        }
    }

    pub fn set_profile_pic(
        &mut self,
        pic: crate::bindings::user_profile_pic,
    ) -> anyhow::Result<()> {
        if unsafe { crate::bindings::user_profile_set_pic(self.as_mut() as *mut _, pic) } != 0 {
            bail!(
                "Failed to set profile pic: {}",
                self.last_error().unwrap_or("Unknown error")
            );
        } else {
            Ok(())
        }
    }

    pub fn accepts_blinded_msgreqs(&self) -> Option<bool> {
        match unsafe {
            crate::bindings::user_profile_get_blinded_msgreqs(self.as_ref() as *const _)
        } {
            -1 => None,
            0 => Some(false),
            _ => Some(true),
        }
    }

    pub fn nts_expiry(&self) -> Option<Timestamp> {
        Timestamp::from_mills(unsafe {
            bindings::user_profile_get_nts_expiry(self.as_ref() as *const _)
        })
    }

    pub fn nts_priority(&self) -> isize {
        unsafe { bindings::user_profile_get_nts_priority(self.as_ref() as *const _) as isize }
    }
}

impl Serialize for super::UserProfileConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("UserProfileConfig", 5)?;
        st.serialize_field("profile_pic", &self.profile_pic())?;
        st.serialize_field("name", self.name())?;
        st.serialize_field("blinded_msgreqs", &self.accepts_blinded_msgreqs())?;
        st.serialize_field("nts_expiry", &self.nts_expiry())?;
        st.serialize_field("nts_priority", &self.nts_priority())?;
        st.end()
    }
}
