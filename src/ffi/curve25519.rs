use std::fmt::Debug;

pub struct Curve25519PublicKey(pub [u8; 32]);
pub struct Curve25519SecretKey(pub [u8; 64]);

pub fn session_curve25519_key_pair() -> Option<(Curve25519PublicKey, Curve25519SecretKey)> {
    let mut public_key = [0u8; 32];
    let mut secret_key = [0u8; 64];

    let result = unsafe {
        super::bindings::session_curve25519_key_pair(
            public_key.as_mut_ptr(),
            secret_key.as_mut_ptr(),
        )
    };

    if result {
        Some((
            Curve25519PublicKey(public_key),
            Curve25519SecretKey(secret_key),
        ))
    } else {
        None
    }
}
