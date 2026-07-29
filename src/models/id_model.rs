use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{errors::AppError, utils::object_id::ObjectId};

#[derive(Debug, Clone)]
pub enum IdType {
    ObjectId(ObjectId),
    String(String),
}

impl Serialize for IdType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

impl<'de> Deserialize<'de> for IdType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(id) => Ok(IdType::String(id)),
            Value::Object(mut map) => match map.remove("$oid") {
                Some(Value::String(id)) => Ok(IdType::String(id)),
                _ => Err(serde::de::Error::custom("Missing $oid field")),
            },
            _ => Err(serde::de::Error::custom("Expected ObjectId-compatible ID")),
        }
    }
}

impl IdType {
    pub fn as_string(&self) -> String {
        match self {
            IdType::ObjectId(oid) => oid.to_hex(),
            IdType::String(s) => s.clone(),
        }
    }

    pub fn from_object_id(oid: ObjectId) -> Self {
        IdType::ObjectId(oid)
    }

    pub fn from_string<S: Into<String>>(s: S) -> Self {
        IdType::String(s.into())
    }

    pub fn to_object_id(&self) -> Result<ObjectId, AppError> {
        match self {
            IdType::ObjectId(oid) => Ok(oid.clone()),
            IdType::String(s) => ObjectId::parse_str(s).map_err(|e| AppError {
                message: format!(
                    "Failed to parse IdType string to ObjectId-compatible ID: {}",
                    e
                ),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{helpers::object_id_helpers, utils::object_id::parse_object_id_value};

    #[test]
    fn id_type_serializes_as_legacy_string() {
        let id = parse_object_id_value("664f2b78a68dff8b9cf934d1").unwrap();
        let json = serde_json::to_value(IdType::from_object_id(id)).unwrap();

        assert_eq!(json, serde_json::json!("664f2b78a68dff8b9cf934d1"));
    }

    #[test]
    fn object_id_helper_accepts_legacy_oid_object() {
        #[derive(Deserialize)]
        struct Payload {
            #[serde(deserialize_with = "object_id_helpers::deserialize_oid")]
            id: ObjectId,
        }

        let payload: Payload = serde_json::from_value(serde_json::json!({
            "id": { "$oid": "664f2b78a68dff8b9cf934d1" }
        }))
        .unwrap();

        assert_eq!(payload.id.to_hex(), "664f2b78a68dff8b9cf934d1");
    }
}
