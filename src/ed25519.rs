use crate::{
    curve25519::{Curve25519PubKey, Curve25519SecKey},
    define_key_type,
};

define_key_type!(ED25519PubKey, 32);
define_key_type!(ED25519SecKey, 64);

impl ED25519PubKey {
    pub fn to_curve25519(&self) -> Curve25519PubKey {
        let mut curve25519_pubkey = [0u8; 32];

        let result = unsafe {
            crate::bindings::session_to_curve25519_pubkey(
                self.as_ref().as_ptr(),
                curve25519_pubkey.as_mut_ptr(),
            )
        };

        if !result {
            panic!("Failed to convert ED25519 to Curve25519");
        }

        curve25519_pubkey.into()
    }
}

impl ED25519SecKey {
    pub fn to_curve25519(&self) -> Curve25519SecKey {
        let mut curve25519_pubkey = [0u8; 32];

        let result = unsafe {
            crate::bindings::session_to_curve25519_seckey(
                self.as_ref().as_ptr(),
                curve25519_pubkey.as_mut_ptr(),
            )
        };

        if !result {
            panic!("Failed to convert ED25519 to Curve25519");
        }

        curve25519_pubkey.into()
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        let mut signature = [0u8; 64];

        let result = unsafe {
            crate::bindings::session_ed25519_sign(
                self.as_ptr(),
                msg.as_ptr(),
                msg.len(),
                signature.as_mut_ptr()
            )
        };

        if !result {
            panic!("Failed to sign message");
        }

        signature
    }

    pub fn seed(&self) -> [u8; 32] {
        let mut seed = [0u8; 32];

        let result = unsafe {
            crate::bindings::session_seed_for_ed_privkey(self.as_ptr(), seed.as_mut_ptr())
        };

        if !result {
            panic!("Failed to get seed from ED25519 private key");
        }

        seed
    }
}

pub fn gen_pair() -> (ED25519PubKey, ED25519SecKey) {
    let mut pubkey = [0u8; 32];
    let mut seckey = [0u8; 64];

    let result = unsafe {
        crate::bindings::session_ed25519_key_pair(pubkey.as_mut_ptr(), seckey.as_mut_ptr())
    };

    if !result {
        panic!("Failed to generate key pair");
    }

    (pubkey.into(), seckey.into())
}

pub fn pad_ed25519_seed(seed: &[u8]) -> [u8; 32] {
    let mut padded_seed = [0u8; 32];
    padded_seed[..seed.len().min(32)].copy_from_slice(seed);
    padded_seed
}

pub fn gen_pair_from_seed(seed: impl Into<[u8; 32]>) -> (ED25519PubKey, ED25519SecKey) {
    let seed = seed.into();
    let mut pubkey = [0u8; 32];
    let mut seckey = [0u8; 64];

    let result = unsafe {
        crate::bindings::session_ed25519_key_pair_seed(
            seed.as_ptr(),
            pubkey.as_mut_ptr(),
            seckey.as_mut_ptr(),
        )
    };

    if !result {
        panic!("Failed to generate key pair");
    }

    (pubkey.into(), seckey.into())
}
