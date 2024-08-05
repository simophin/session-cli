use std::borrow::Cow;

use rusqlite::{types::FromSql, ToSql};
use serde::{Deserialize, Serialize};

use crate::{
    ed25519::{ED25519PubKey, ED25519SecKey},
    identity::Identity,
    network::swarm::SwarmAuth,
};

use super::AppSetting;

impl AppSetting for Identity {
    const NAME: &'static str = "identity";
}

#[derive(Serialize, Deserialize)]
struct IdentityValue<'a> {
    ed25519_pub_key: Cow<'a, str>,
    ed25519_sec_key: Cow<'a, str>,
    session_id: Cow<'a, str>,
}

impl ToSql for Identity {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(
                serde_json::to_string(&IdentityValue {
                    ed25519_pub_key: Cow::Borrowed(self.ed25519_pub_key().hex()),
                    ed25519_sec_key: Cow::Borrowed(self.ed25519_sec_key().hex()),
                    session_id: Cow::Borrowed(self.session_id().as_str()),
                })
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            ),
        ))
    }
}

impl FromSql for Identity {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value: IdentityValue = serde_json::from_str(value.as_str()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))?;

        Ok(Identity::new((
            ED25519PubKey::from_hex(value.ed25519_pub_key.into_owned().as_str())
                .map_err(|e| rusqlite::types::FromSqlError::Other(e.into()))?,
            ED25519SecKey::from_hex(value.ed25519_sec_key.into_owned().as_str())
                .map_err(|e| rusqlite::types::FromSqlError::Other(e.into()))?,
        )))
    }
}
