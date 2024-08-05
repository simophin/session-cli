use crate::define_key_type;

define_key_type!(Curve25519PubKey, 32);
define_key_type!(Curve25519SecKey, 32);

pub fn gen_pair() -> (Curve25519PubKey, Curve25519SecKey) {
    let mut pubkey = [0u8; 32];
    let mut seckey = [0u8; 32];

    let result = unsafe {
        crate::bindings::session_curve25519_key_pair(pubkey.as_mut_ptr(), seckey.as_mut_ptr())
    };

    if !result {
        panic!("Failed to generate key pair");
    }

    (pubkey.into(), seckey.into())
}

impl Curve25519SecKey {
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let mut signature = [0u8; 64];

        let result = unsafe {
            crate::bindings::session_xed25519_sign(
                signature.as_mut_ptr(),
                self.as_ptr(),
                message.as_ptr(),
                message.len(),
            )
        };

        if !result {
            panic!("Failed to sign message");
        }

        signature
    }
}
