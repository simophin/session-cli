use crate::bindings;
use crate::curve25519::{Curve25519PubKey, Curve25519SecKey};
use crate::cwrapper::{CArrayWrapper, CWrapper};
use crate::ed25519::ED25519PubKey;
use crate::utils::StringExt;
use std::net::Ipv4Addr;
use std::ptr::null_mut;
use url::Url;

pub struct OnionRequestBuilder(CWrapper<bindings::onion_request_builder_object>);

impl OnionRequestBuilder {
    pub fn new() -> Self {
        let mut instance = null_mut();

        unsafe {
            bindings::onion_request_builder_init(&mut instance);
            bindings::onion_request_builder_set_enc_type(
                instance,
                bindings::ENCRYPT_TYPE_ENCRYPT_TYPE_X_CHA_CHA_20,
            );
        }

        Self(
            CWrapper::new_with_destroyer(instance, bindings::onion_request_builder_free)
                .expect("Failed to create OnionRequestBuilder"),
        )
    }

    pub fn set_snode_destination(
        &mut self,
        _ip: Ipv4Addr,
        _port: u16,
        snode_pub_key: &ED25519PubKey,
        snode_pub_key_curve: &Curve25519PubKey,
    ) -> &mut Self {
        unsafe {
            bindings::onion_request_builder_set_snode_destination(
                self.0.as_mut_ptr(),
                snode_pub_key.hex_cstr().as_ptr(),
                snode_pub_key_curve.hex_cstr().as_ptr(),
            );
        }

        self
    }

    pub fn set_server_destination(
        &mut self,
        url: &Url,
        _method: &http::Method,
        pub_key: &Curve25519PubKey,
    ) -> Result<&mut Self, &'static str> {
        unsafe {
            bindings::onion_request_builder_set_server_destination(
                self.0.as_mut_ptr(),
                url.host_str()
                    .ok_or("Host not specified")?
                    .to_cstr()
                    .as_ref()
                    .as_ptr(),
                url.path().to_cstr().as_ref().as_ptr(),
                url.scheme().to_cstr().as_ref().as_ptr(),
                url.port_or_known_default()
                    .ok_or("Port not given or unable to infer")?,
                pub_key.hex_cstr().as_ptr(),
            )
        };

        Ok(self)
    }

    pub fn add_hop(&mut self, snode_pub_key: (&Curve25519PubKey, &ED25519PubKey)) -> &mut Self {
        unsafe {
            bindings::onion_request_builder_add_hop(
                self.0.as_mut_ptr(),
                snode_pub_key.1.hex_cstr().as_ptr(),
                snode_pub_key.0.hex_cstr().as_ptr(),
            );
        }

        self
    }

    pub fn build(
        &mut self,
        in_bytes: &[u8],
    ) -> Result<(impl AsRef<[u8]>, Curve25519PubKey, Curve25519SecKey), &'static str> {
        let mut payload = null_mut();
        let mut payload_len = 0;

        let mut pub_key = [0u8; 32];
        let mut sec_key = [0u8; 32];

        let r = unsafe {
            bindings::onion_request_builder_build(
                self.0.as_mut_ptr(),
                in_bytes.as_ptr(),
                in_bytes.len(),
                &mut payload,
                &mut payload_len,
                pub_key.as_mut_ptr(),
                sec_key.as_mut_ptr(),
            )
        };

        if !r {
            return Err("Failed to build OnionRequest");
        }

        CArrayWrapper::new(payload, payload_len)
            .ok_or("builder's output is null")
            .map(|payload| {
                (
                    payload,
                    Curve25519PubKey::from(pub_key),
                    Curve25519SecKey::from(sec_key),
                )
            })
    }
}

pub fn decrypt_onion_response(
    cipher: &[u8],
    dest_pub_key: &Curve25519PubKey,
    final_pub_key: &Curve25519PubKey,
    final_sec_key: &Curve25519SecKey,
) -> Option<impl AsRef<[u8]>> {
    let mut plaintext = null_mut();
    let mut plaintext_len = 0;
    let r = unsafe {
        bindings::onion_request_decrypt(
            cipher.as_ptr(),
            cipher.len(),
            bindings::ENCRYPT_TYPE_ENCRYPT_TYPE_X_CHA_CHA_20,
            dest_pub_key.as_ptr() as *mut _,
            final_pub_key.as_ptr() as *mut _,
            final_sec_key.as_ptr() as *mut _,
            &mut plaintext,
            &mut plaintext_len,
        )
    };

    if !r {
        return None;
    }

    CArrayWrapper::new(plaintext, plaintext_len)
}
