use base64::{prelude::BASE64_STANDARD, Engine};
use derive_more::Deref;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::ToSql;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deref, Debug, Default)]
pub struct Base64<T>(pub T);

impl<'de> Deserialize<'de> for Base64<Vec<u8>> {
    fn deserialize<D>(deserializer: D) -> Result<Base64<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = BASE64_STANDARD
            .decode(s.as_bytes())
            .map_err(serde::de::Error::custom)?;
        Ok(Self(bytes))
    }
}

impl<T: AsRef<[u8]>> Serialize for Base64<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = BASE64_STANDARD.encode(self.0.as_ref());
        serializer.serialize_str(&s)
    }
}

impl FromSql for Base64<Vec<u8>> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let bytes = <String as FromSql>::column_result(value)?;
        BASE64_STANDARD
            .decode(bytes)
            .map_err(|e| FromSqlError::Other(Box::new(e)))
            .map(Self)
    }
}

impl<T: AsRef<[u8]>> ToSql for Base64<T> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let s = BASE64_STANDARD.encode(self.0.as_ref());
        Ok(ToSqlOutput::from(s))
    }
}
