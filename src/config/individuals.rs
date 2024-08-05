use super::Config;
use crate::bindings;
use crate::cwrapper::CWrapper;
use crate::ed25519::ED25519SecKey;
use anyhow::Context;
use std::ffi::{c_char, c_int, c_uchar, CStr};
use std::ptr::{null, null_mut};

pub trait IndividualConfig: From<CWrapper<bindings::config_object>> + Config {
    unsafe fn c_new(
        obj: *mut *mut bindings::config_object,
        ed25519_sec_key: *const c_uchar,
        dump: *const c_uchar,
        dump_len: usize,
        error: *mut c_char,
    ) -> c_int;

    fn new(sec_key: &ED25519SecKey, dump: Option<&[u8]>) -> anyhow::Result<Self> {
        let dump = dump.unwrap_or_default();

        let mut config = null_mut();
        let mut error = [0i8; 256];
        let res = unsafe {
            Self::c_new(
                &mut config,
                sec_key.as_ptr(),
                if dump.is_empty() {
                    null()
                } else {
                    dump.as_ptr()
                },
                dump.len(),
                error.as_mut_ptr(),
            )
        };

        if res == 0 {
            CWrapper::new_with_destroyer(config, bindings::config_free)
                .map(Into::into)
                .context("Empty config")
        } else {
            let error = unsafe { CStr::from_ptr(error.as_ptr()) }
                .to_str()
                .unwrap_or("Invalid UTF-8");
            Err(anyhow::anyhow!("Failed to create config: {}", error))
        }
    }
}

macro_rules! define_individual_config_type {
    ($name:ident, $init_func:ident) => {
        crate::define_config_type!($name);

        impl IndividualConfig for $name {
            unsafe fn c_new(
                obj: *mut *mut crate::bindings::config_object,
                ed25519_sec_key: *const std::ffi::c_uchar,
                dump: *const std::ffi::c_uchar,
                dump_len: usize,
                error: *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                crate::bindings::$init_func(obj, ed25519_sec_key, dump, dump_len, error)
            }
        }
    };
}

define_individual_config_type!(UserGroupsConfig, user_groups_init);
define_individual_config_type!(UserProfileConfig, user_profile_init);
define_individual_config_type!(ContactsConfig, contacts_init);
define_individual_config_type!(ConvoInfoVolatileConfig, convo_info_volatile_init);
