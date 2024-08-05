use crate::base64::Base64;
use crate::crypto::decrypt_incoming;
use crate::curve25519::Curve25519SecKey;
use crate::ed25519::ED25519PubKey;
use crate::ed25519::{self, ED25519SecKey};
use crate::mnemonic::ENGLISH;
use crate::network::swarm::SwarmAuth;
use crate::session_id::IndividualID;
use anyhow::anyhow;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq)]
pub struct Identity {
    sec_key: Curve25519SecKey,
    ed25519_key_pair: (ED25519PubKey, ED25519SecKey),
    session_id: IndividualID,
}

impl Debug for Identity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.session_id.fmt(f)
    }
}

impl Identity {
    pub fn new(ed25519_key_pair: (ED25519PubKey, ED25519SecKey)) -> Self {
        let sec_key = ed25519_key_pair.1.to_curve25519();
        let session_id = IndividualID::new(ed25519_key_pair.0.to_curve25519());
        Self {
            sec_key,
            session_id,
            ed25519_key_pair,
        }
    }

    pub fn from_mnemonic(mnemonic: &str) -> anyhow::Result<Self> {
        let seed =
            hex::decode(crate::mnemonic::decode(&mnemonic, &ENGLISH).expect("To decode mnemonic"))
                .expect("To decode mnemonic hex");

        let ed25519_key_pair = ed25519::gen_pair_from_seed(ed25519::pad_ed25519_seed(&seed));

        Ok(Self {
            sec_key: ed25519_key_pair.1.to_curve25519(),
            session_id: IndividualID::new(ed25519_key_pair.0.to_curve25519()),
            ed25519_key_pair,
        })
    }

    pub fn mnemonic(&self) -> String {
        crate::mnemonic::encode(&hex::encode(self.ed25519_sec_key().seed()), &ENGLISH)
    }

    pub fn sec_key(&self) -> &Curve25519SecKey {
        &self.sec_key
    }

    pub fn ed25519_pub_key(&self) -> &ED25519PubKey {
        &self.ed25519_key_pair.0
    }

    pub fn ed25519_sec_key(&self) -> &ED25519SecKey {
        &self.ed25519_key_pair.1
    }

    pub fn gen() -> Self {
        let ed25519_key_pair = ed25519::gen_pair();
        Self {
            sec_key: ed25519_key_pair.1.to_curve25519(),
            session_id: IndividualID::new(ed25519_key_pair.0.to_curve25519()),
            ed25519_key_pair,
        }
    }
}

#[derive(Serialize)]
struct IdentitySignature {
    signature: Base64<[u8; 64]>,
}

impl SwarmAuth for Identity {
    type SessionIDType = IndividualID;

    fn sign(&self, payload: &[u8]) -> Option<impl Serialize + 'static> {
        Some(IdentitySignature {
            signature: Base64(self.ed25519_sec_key().sign(payload)),
        })
    }

    fn decrypt(
        &self,
        payload: &[u8],
    ) -> anyhow::Result<(IndividualID, impl AsRef<[u8]> + 'static)> {
        let (id, buf) = decrypt_incoming(self.ed25519_sec_key(), payload)?;
        Ok((id.try_into().map_err(|_| anyhow!("Invalid ID type"))?, buf))
    }

    fn session_id(&self) -> Cow<IndividualID> {
        Cow::Borrowed(&self.session_id)
    }

    fn ed25519_pub_key(&self) -> Cow<ED25519PubKey> {
        Cow::Borrowed(&self.ed25519_key_pair.0)
    }
}
