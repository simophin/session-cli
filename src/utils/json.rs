use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T: Serialize> Serialize for Json<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_json::to_string(&self.0)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Json<T>
where
    T: for<'d> Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Json<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        serde_json::from_str(&value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
