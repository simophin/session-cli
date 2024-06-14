use std::fmt::Debug;

pub struct ED25519PublicKey(pub [u8; 32]);
pub struct ED25519SecretKey(pub [u8; 64]);


pub fn session_ed25519_key_pair() -> Option<(ED25519PublicKey, ED25519SecretKey)> {
    let mut public_key = [0u8; 32];
    let mut secret_key = [0u8; 64];

    let result = unsafe {
        super::bindings::session_ed25519_key_pair(public_key.as_mut_ptr(), secret_key.as_mut_ptr())
    };

    if result {
        Some((ED25519PublicKey(public_key), ED25519SecretKey(secret_key)))
    } else {
        None
    }
}