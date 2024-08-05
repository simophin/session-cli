use anyhow::Context;

use crate::{bindings, cwrapper::CArrayWrapper, session_id::BlindedID};

pub fn blinded_ids(
    session_id: &str,
    server_pub_key_hex: &str,
) -> anyhow::Result<(BlindedID, BlindedID)> {
    let bindings::blinded_ids {
        id1,
        id1_len,
        id2,
        id2_len,
    } = unsafe {
        bindings::session_create_blind15_id(
            session_id.as_ptr() as *const _,
            session_id.len(),
            server_pub_key_hex.as_ptr() as *const _,
            server_pub_key_hex.len(),
        )
    };

    let id1 = CArrayWrapper::new(id1 as *mut u8, id1_len).context("Invalid ID1")?;
    let id2 = CArrayWrapper::new(id2 as *mut u8, id2_len).context("Invalid ID2")?;

    let id1 = std::str::from_utf8(id1.as_slice())?;
    let id2 = std::str::from_utf8(id2.as_slice())?;

    Ok((id1.parse()?, id2.parse()?))
}

#[cfg(test)]
mod tests {
    use crate::{curve25519::gen_pair, ed25519, identity::Identity, network::swarm::SwarmAuth};

    use super::*;

    #[test]
    fn blinded_ids_works() {
        let identity = Identity::new(ed25519::gen_pair());
        let (server_pub_key, _) = gen_pair();
        let server_pub_key = hex::encode(server_pub_key.as_ref());

        let (id1, id2) = blinded_ids(identity.session_id().as_str(), &server_pub_key)
            .expect("To generate blinded IDs");

        println!("ID1 = {id1}, ID2 = {id2}");
    }
}
