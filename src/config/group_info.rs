use super::{ConfigExt, GroupInfoConfig};
use crate::bindings;
use crate::clock::Timestamp;
use crate::utils::StringExt;
use anyhow::bail;
use std::ffi::CStr;

impl GroupInfoConfig {
    pub fn name(&self) -> &str {
        unsafe {
            let name = bindings::groups_info_get_name(self.as_ref() as *const _);
            CStr::from_ptr(name).to_str().unwrap_or_default()
        }
    }

    pub fn set_name(&mut self, name: &str) -> anyhow::Result<()> {
        if unsafe {
            bindings::groups_info_set_name(
                self.as_mut() as *mut _,
                name.to_cstr().as_ref().as_ptr(),
            )
        } != 0
        {
            bail!(
                "Error setting name: {}",
                self.last_error().unwrap_or("Unknown error")
            )
        }

        Ok(())
    }

    pub fn profile_pic(&self) -> bindings::user_profile_pic {
        unsafe { bindings::groups_info_get_pic(self.as_ref() as *const _) }
    }

    pub fn set_profile_pic(&mut self, pic: bindings::user_profile_pic) -> anyhow::Result<()> {
        if unsafe { bindings::groups_info_set_pic(self.as_mut() as *mut _, pic) } != 0 {
            bail!(
                "Error setting profile pic: {}",
                self.last_error().unwrap_or("Unknown error")
            )
        }

        Ok(())
    }

    pub fn created(&self) -> Option<Timestamp> {
        let created = unsafe { bindings::groups_info_get_created(self.as_ref() as *const _) };
        if created == 0 {
            None
        } else {
            Timestamp::from_mills(created)
        }
    }
}
