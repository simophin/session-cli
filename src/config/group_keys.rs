use crate::base64::Base64;
use crate::bindings;
use crate::bindings::seqno_t;
use crate::config::{GroupAuthData, GroupInfoConfig, GroupMemberConfig};
use crate::cwrapper::{CArrayWrapper, CWrapper};
use crate::ed25519::{ED25519PubKey, ED25519SecKey};
use crate::oxen_api::retrieve::Message;
use crate::session_id::SessionID;
use crate::utils::{CArrayExt, StringExt};
use anyhow::{bail, Context};
use serde::Serialize;
use serde_json::Value;
use std::ffi::{c_uchar, CStr};
use std::ptr::{null, null_mut};

pub struct GroupKeys {
    wrapper: CWrapper<bindings::config_group_keys>,
    group_pub_key: ED25519PubKey,
}

pub type AdminKey = [u8; 32];

#[derive(Serialize)]
pub struct SubaccountAuth {
    pub subaccount: Base64<[u8; 36]>,
    pub subaccount_sig: Base64<[u8; 64]>,
    pub signature: Base64<[u8; 64]>,
}

impl Default for SubaccountAuth {
    fn default() -> Self {
        Self {
            subaccount: Base64([0u8; 36]),
            subaccount_sig: Base64([0u8; 64]),
            signature: Base64([0u8; 64]),
        }
    }
}

impl GroupKeys {
    pub fn new(
        user_key: &ED25519SecKey,
        group_pub_key: &ED25519PubKey,
        group_sec_key: Option<&ED25519SecKey>,
        group_info_config: &mut GroupInfoConfig,
        group_members_config: &mut GroupMemberConfig,
        dump: &[u8],
    ) -> anyhow::Result<Self> {
        let mut error = [0u8; 256];
        let mut instance = null_mut();
        let rc = unsafe {
            bindings::groups_keys_init(
                &mut instance,
                user_key.as_ptr(),
                group_pub_key.as_ptr(),
                group_sec_key.map_or(null_mut(), |k| k.as_ptr()),
                group_info_config.as_mut() as *mut _,
                group_members_config.as_mut() as *mut _,
                if dump.is_empty() {
                    null()
                } else {
                    dump.as_ptr()
                },
                dump.len(),
                error.as_mut_ptr() as *mut _,
            )
        };

        if rc != 0 {
            return Err(anyhow::anyhow!(
                "groups_keys_init failed: {}",
                error.cstr_to_str().unwrap_or("Unknown error")
            ));
        }

        CWrapper::new_with_destroyer(instance, bindings::groups_keys_free)
            .map(|wrapper| Self {
                wrapper,
                group_pub_key: group_pub_key.clone(),
            })
            .context("Failed to create GroupKeys instance")
    }

    pub fn group_pub_key(&self) -> &ED25519PubKey {
        &self.group_pub_key
    }

    pub fn encrypt_message(&self, plaintext: &[u8]) -> Option<impl AsRef<[u8]>> {
        let mut cipher_text = null_mut();
        let mut cipher_text_len = 0;
        unsafe {
            bindings::groups_keys_encrypt_message(
                self.wrapper.as_ptr(),
                plaintext.as_ptr(),
                plaintext.len(),
                &mut cipher_text,
                &mut cipher_text_len,
            )
        };

        CArrayWrapper::new(cipher_text, cipher_text_len)
    }

    pub fn decrypt_message(
        &self,
        cipher_text: &[u8],
    ) -> Option<(SessionID, impl AsRef<[u8]> + 'static)> {
        let mut plaintext = null_mut();
        let mut plaintext_len = 0;
        let mut session_id = [0u8; 67];
        let r = unsafe {
            bindings::groups_keys_decrypt_message(
                self.wrapper.as_mut_ptr(),
                cipher_text.as_ptr(),
                cipher_text.len(),
                session_id.as_mut_ptr() as *mut _,
                &mut plaintext,
                &mut plaintext_len,
            )
        };

        if !r {
            return None;
        }

        let session_id = std::str::from_utf8(&session_id[..66]).ok()?.parse().ok()?;
        CArrayWrapper::new(plaintext, plaintext_len).map(|plaintext| (session_id, plaintext))
    }

    pub fn sub_key_sign(
        &self,
        data: &[u8],
        auth_data: &GroupAuthData,
    ) -> anyhow::Result<SubaccountAuth> {
        let mut auth = SubaccountAuth::default();

        // Copy the c buffer so we can mutate on it
        let mut instance = *self.wrapper.as_ref();

        let r = unsafe {
            bindings::groups_keys_swarm_subaccount_sign_binary(
                &mut instance,
                data.as_ptr(),
                data.len(),
                auth_data.as_ptr(),
                auth.subaccount.0.as_mut_ptr(),
                auth.subaccount_sig.0.as_mut_ptr(),
                auth.signature.0.as_mut_ptr(),
            )
        };

        if !r {
            bail!("Error signing using sub-key: {}", unsafe {
                CStr::from_ptr(instance.last_error as *const _)
                    .to_str()
                    .unwrap_or("Unknown error")
            })
        }

        Ok(auth)
    }

    pub fn is_admin(&self) -> bool {
        unsafe { bindings::groups_keys_is_admin(self.wrapper.as_ptr()) }
    }

    pub fn rekey(
        &mut self,
        info: &mut GroupInfoConfig,
        members: &mut GroupMemberConfig,
    ) -> anyhow::Result<&[u8]> {
        let mut out = null();
        let mut out_len = 0;
        let success = unsafe {
            bindings::groups_keys_rekey(
                self.wrapper.as_mut(),
                info.as_mut(),
                members.as_mut(),
                &mut out,
                &mut out_len,
            )
        };

        if !success {
            bail!("Failed to rekey group");
        }

        Ok(unsafe { std::slice::from_raw_parts(out, out_len) })
    }

    fn pending_config(&self) -> Option<CArrayWrapper<u8>> {
        let mut data = null();
        let mut data_len = 0;
        if !unsafe {
            bindings::groups_keys_pending_config(self.wrapper.as_ptr(), &mut data, &mut data_len)
        } {
            return None;
        }

        CArrayWrapper::new(data as *mut c_uchar, data_len)
    }
}

impl super::NamedConfig for GroupKeys {
    const CONFIG_TYPE_NAME: &'static str = "GroupKeys";
}

impl super::Config for GroupKeys {
    type MergeArg<'a> = (&'a mut GroupInfoConfig, &'a mut GroupMemberConfig);
    type PushData = Option<CArrayWrapper<u8>>;

    fn merge<'a>(
        &mut self,
        messages: &'a [Message],
        (info, members): Self::MergeArg<'a>,
    ) -> Result<usize, String> {
        let mut count = 0;
        for msg in messages {
            if unsafe {
                bindings::groups_keys_load_message(
                    self.wrapper.as_mut_ptr(),
                    msg.hash.as_str().to_cstr().as_ref().as_ptr(),
                    msg.data.as_ptr(),
                    msg.data.len(),
                    msg.created.as_millis() as i64,
                    info.as_mut() as *mut _,
                    members.as_mut() as *mut _,
                )
            } {
                count += 1;
            }
        }

        Ok(count)
    }

    fn current_hashes(&self) -> Vec<String> {
        let hashes = unsafe { bindings::groups_keys_current_hashes(self.wrapper.as_ptr()) };
        CWrapper::new(hashes)
            .iter()
            .flat_map(|c| c.slice().into_iter().map(|c| unsafe { CStr::from_ptr(*c) }))
            .filter_map(|s| s.to_str().ok())
            .map(|s| s.to_string())
            .collect()
    }

    fn push(&mut self) -> anyhow::Result<Self::PushData> {
        Ok(self.pending_config())
    }

    fn confirm_pushed(&mut self, _seq: seqno_t, _msg_hash: &str) {}

    fn needs_push(&self) -> bool {
        self.pending_config().is_some()
    }

    fn needs_dump(&self) -> bool {
        unsafe { bindings::groups_keys_needs_dump(self.wrapper.as_ptr()) }
    }

    fn dump(&mut self) -> Option<impl AsRef<[u8]> + 'static> {
        let mut data = null_mut();
        let mut data_len = 0;
        unsafe {
            bindings::groups_keys_dump(self.wrapper.as_mut_ptr(), &mut data, &mut data_len);
        }

        CArrayWrapper::new(data as *mut c_uchar, data_len)
    }

    fn to_json(&self) -> anyhow::Result<Value> {
        Ok(Value::Object(Default::default()))
    }
}
