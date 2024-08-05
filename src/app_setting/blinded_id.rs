use rusqlite::{types::FromSql, ToSql};

use crate::session_id::BlindedID;

use super::AppSetting;

impl AppSetting for BlindedID {
    const NAME: &'static str = "blinded_id";
}

impl FromSql for BlindedID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(value
            .as_str()?
            .parse()
            .map_err(|e: anyhow::Error| rusqlite::types::FromSqlError::Other(e.into()))?)
    }
}

impl ToSql for BlindedID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Text(self.as_str().as_bytes()),
        ))
    }
}
