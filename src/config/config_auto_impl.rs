use crate::bindings;
use crate::config::{Config, NamedConfig};
use crate::cwrapper::{CArrayWrapper, CWrapper};
use crate::oxen_api::retrieve::Message;
use anyhow::Context;
use std::ffi::{c_char, CStr, CString};
use std::ptr::null_mut;

impl bindings::config_string_list {
    pub(super) fn slice(&self) -> &[*mut c_char] {
        unsafe { std::slice::from_raw_parts(self.value, self.len) }
    }
}

impl<T: AsMut<bindings::config_object> + AsRef<bindings::config_object> + NamedConfig> Config
    for T
{
    type MergeArg<'a> = ();
    type PushData = CWrapper<bindings::config_push_data>;

    fn merge<'a>(
        &mut self,
        messages: &'a [Message],
        _arg: Self::MergeArg<'a>,
    ) -> Result<usize, String> {
        let mut num_merged = 0;
        let mut error = [0u8; 256];

        for msg in messages {
            if unsafe {
                bindings::session_config_merge(
                    self.as_mut(),
                    msg.data.as_ptr(),
                    msg.data.len(),
                    msg.hash.as_ptr() as *const _,
                    msg.hash.len(),
                    error.as_mut_ptr() as *mut _,
                    error.len(),
                )
            } {
                num_merged += 1;
            } else {
                let error = CStr::from_bytes_until_nul(error.as_ref())
                    .map_err(|_| "Invalid error message")?
                    .to_str()
                    .map_err(|_| "Invalid UTF-8 in error message")?;

                return Err(error.to_string());
            }
        }

        return Ok(num_merged);
    }

    fn current_hashes(&self) -> Vec<String> {
        let hashes = unsafe { bindings::config_current_hashes(self.as_ref()) };

        CWrapper::new(hashes)
            .iter()
            .flat_map(|c| c.slice().into_iter().map(|c| unsafe { CStr::from_ptr(*c) }))
            .filter_map(|s| s.to_str().ok())
            .map(|s| s.to_string())
            .collect()
    }

    fn push(&mut self) -> anyhow::Result<Self::PushData> {
        CWrapper::new(unsafe { bindings::config_push(self.as_mut() as *mut _) })
            .context("Empty push data")
    }

    fn confirm_pushed(&mut self, seq: bindings::seqno_t, msg_hash: &str) {
        let msg_hash = CString::new(msg_hash).unwrap();
        unsafe {
            bindings::config_confirm_pushed(self.as_mut() as *mut _, seq, msg_hash.as_ptr());
        }
    }

    fn needs_push(&self) -> bool {
        unsafe { bindings::config_needs_push(self.as_ref() as *const _) }
    }

    fn needs_dump(&self) -> bool {
        unsafe { bindings::config_needs_dump(self.as_ref() as *const _) }
    }

    fn dump(&mut self) -> Option<impl AsRef<[u8]> + 'static> {
        let mut out = null_mut();
        let mut len = 0;
        unsafe { bindings::config_dump(self.as_mut() as *mut _, &mut out, &mut len) };
        CArrayWrapper::new(out, len)
    }

    fn to_json(&self) -> anyhow::Result<serde_json::Value> {
        let json = unsafe { bindings::session_config_dump_json(self.as_ref() as *const _) };
        let json = CWrapper::new(json as *mut c_char).context("Empty json")?;
        unsafe { CStr::from_ptr(json.as_ptr()) }
            .to_str()
            .context("Invalid UTF-8 in json")
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).context("Invalid JSON"))
    }
}

impl<C: Config + AsRef<bindings::config_object>> super::ConfigExt for C {
    fn last_error(&self) -> Option<&str> {
        if self.as_ref().last_error.is_null() {
            return None;
        }

        unsafe { CStr::from_ptr(self.as_ref().last_error) }
            .to_str()
            .ok()
    }
}

impl bindings::config_push_data {
    pub fn config_data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.config, self.config as usize) }
    }

    pub fn obsolete_message_hashes(&self) -> impl Iterator<Item = &CStr> {
        unsafe {
            std::slice::from_raw_parts(self.obsolete, self.obsolete_len)
                .iter()
                .map(|&ptr| CStr::from_ptr(ptr))
        }
    }
}
