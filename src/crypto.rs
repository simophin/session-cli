use std::ptr::null_mut;

use anyhow::{bail, Context};

use crate::session_id::SessionID;
use crate::{
    bindings, curve25519::Curve25519PubKey, cwrapper::CArrayWrapper, ed25519::ED25519SecKey,
};

pub fn encrypt_for_recipient(
    recipient_pub_key: &Curve25519PubKey,
    sender_key: &ED25519SecKey,
    plaintext: impl AsRef<[u8]>,
) -> Vec<u8> {
    let plaintext = plaintext.as_ref();
    let mut cipher = null_mut::<u8>();
    let mut cipher_len = 0;
    let r = unsafe {
        bindings::session_encrypt_for_recipient_deterministic(
            plaintext.as_ptr(),
            plaintext.len(),
            sender_key.as_ptr(),
            recipient_pub_key.as_ptr(),
            &mut cipher,
            &mut cipher_len,
        )
    };

    assert_eq!(r, true);

    CArrayWrapper::new(cipher, cipher_len)
        .map(|s| s.as_slice().to_vec())
        .unwrap_or_default()
}

pub fn decrypt_incoming(
    receiver_key: &ED25519SecKey,
    ciphertext: impl AsRef<[u8]>,
) -> anyhow::Result<(SessionID, Vec<u8>)> {
    let mut plaintext = null_mut::<u8>();
    let mut plaintext_len = 0;

    let ciphertext = ciphertext.as_ref();

    let mut session_id = vec![0u8; 67];

    let r = unsafe {
        bindings::session_decrypt_incoming(
            ciphertext.as_ptr(),
            ciphertext.len(),
            receiver_key.as_ptr(),
            session_id.as_mut_ptr() as *mut _,
            &mut plaintext,
            &mut plaintext_len,
        )
    };

    if !r {
        bail!("Failed to decrypt incoming message");
    }

    let plaintext = CArrayWrapper::new(plaintext, plaintext_len)
        .map(|s| s.as_slice().to_vec())
        .unwrap_or_default();

    session_id.pop();
    let session_id = std::str::from_utf8(&session_id)
        .context("Session ID is not valid UTF-8")?
        .parse()
        .context("Failed to parse session ID")?;

    Ok((session_id, plaintext))
}
